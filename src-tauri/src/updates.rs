use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::{Update, UpdaterExt};

pub const APP_UPDATE_STATE_EVENT: &str = "worth://updates/state";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum AppUpdateCheckModeDto {
    Startup,
    User,
}

/// Complete snapshot of Worth's update system.
///
/// This is returned from update IPC commands and emitted over
/// `worth://updates/state` whenever the update state changes.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AppUpdateStateDto {
    /// Version of the currently running app.
    pub current_version: String,
    /// Current state-machine variant for checking, downloading, installing, or errors.
    pub status: AppUpdateStatusDto,
    /// Timestamp of the most recent attempted/completed check, when known.
    pub checked_at: Option<String>,
    /// Timestamp of the most recent state change.
    pub updated_at: String,
    /// Monotonic counter incremented on every state change.
    pub revision: u32,
    /// Whether this installation should expose update actions.
    pub supports_updates: bool,
}

/// Update state machine exposed to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AppUpdateStatusDto {
    /// No check has happened in this app session.
    Idle,
    /// Worth is checking whether a newer version is available.
    Checking { check_mode: AppUpdateCheckModeDto },
    /// A check completed and no newer version was available.
    UpToDate { check_mode: AppUpdateCheckModeDto },
    /// An update is being downloaded.
    Downloading {
        check_mode: AppUpdateCheckModeDto,
        update: AppUpdateMetadataDto,
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
    },
    /// A downloaded update is being installed.
    Installing {
        check_mode: AppUpdateCheckModeDto,
        update: AppUpdateMetadataDto,
    },
    /// An update was downloaded and is pending installation.
    Downloaded {
        check_mode: AppUpdateCheckModeDto,
        update: AppUpdateMetadataDto,
    },
    /// An update was installed and a restart is needed or in progress.
    Installed {
        check_mode: AppUpdateCheckModeDto,
        update: AppUpdateMetadataDto,
    },
    /// A check, download, or install step failed.
    Error {
        check_mode: Option<AppUpdateCheckModeDto>,
        phase: AppUpdatePhaseDto,
        code: AppUpdateErrorCodeDto,
        message: String,
        update: Option<AppUpdateMetadataDto>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AppUpdateMetadataDto {
    pub version: String,
    pub current_version: String,
    pub target: String,
    pub body: Option<String>,
    pub date: Option<String>,
}

/// Update operation phase that produced an error.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum AppUpdatePhaseDto {
    Checking,
    Downloading,
    Installing,
}

/// Stable error categories used by the frontend for friendly update messages.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum AppUpdateErrorCodeDto {
    /// Updater configuration is missing or invalid.
    Configuration,
    /// The update manifest could not be parsed or did not match the expected format.
    Manifest,
    /// The update service could not be reached or the release was not found.
    Network,
    /// The downloaded update could not be authenticated.
    Signature,
    /// The platform installer or file replacement step failed.
    Install,
    /// The current platform or target is not supported for updater operations.
    Unsupported,
    /// Install was requested without a downloaded pending update.
    NoPendingUpdate,
    /// Fallback for updater errors that do not fit a known bucket.
    Unknown,
}

#[derive(Clone)]
pub struct AppUpdateManager {
    inner: Arc<Mutex<AppUpdateInner>>,
    busy: Arc<AtomicBool>,
}

struct AppUpdateInner {
    state: AppUpdateStateDto,
    pending_update: Option<PendingAppUpdate>,
}

struct PendingAppUpdate {
    update: Box<Update>,
    bytes: Vec<u8>,
    metadata: AppUpdateMetadataDto,
    check_mode: AppUpdateCheckModeDto,
}

struct BusyGuard {
    busy: Arc<AtomicBool>,
}

impl Drop for BusyGuard {
    fn drop(&mut self) {
        self.busy.store(false, Ordering::Release);
    }
}

impl AppUpdateManager {
    pub fn new(current_version: String) -> Self {
        Self {
            inner: Arc::new(Mutex::new(AppUpdateInner {
                state: AppUpdateStateDto {
                    current_version,
                    status: AppUpdateStatusDto::Idle,
                    checked_at: None,
                    updated_at: timestamp(),
                    revision: 0,
                    supports_updates: !cfg!(debug_assertions)
                        && tauri_plugin_updater::target().is_some(),
                },
                pending_update: None,
            })),
            busy: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn state(&self) -> AppUpdateStateDto {
        self.inner
            .lock()
            .expect("update state lock poisoned")
            .state
            .clone()
    }

    pub fn check_on_startup(&self, app: AppHandle) {
        if !self.state().supports_updates {
            return;
        }

        let updates = self.clone();
        tauri::async_runtime::spawn(async move {
            let _ = updates
                .check_for_updates_inner(app, AppUpdateCheckModeDto::Startup)
                .await;
        });
    }

    pub async fn check_for_updates(&self, app: AppHandle) -> AppUpdateStateDto {
        self.check_for_updates_inner(app, AppUpdateCheckModeDto::User)
            .await
    }

    async fn check_for_updates_inner(
        &self,
        app: AppHandle,
        mode: AppUpdateCheckModeDto,
    ) -> AppUpdateStateDto {
        let Some(_guard) = self.begin_operation() else {
            return self.state();
        };

        self.inner
            .lock()
            .expect("update state lock poisoned")
            .pending_update = None;
        self.set_status(
            &app,
            AppUpdateStatusDto::Checking { check_mode: mode },
            None,
        );
        let checked_at = timestamp();

        if !self.state().supports_updates {
            return self.set_status(
                &app,
                AppUpdateStatusDto::Error {
                    check_mode: Some(mode),
                    phase: AppUpdatePhaseDto::Checking,
                    code: AppUpdateErrorCodeDto::Unsupported,
                    message: "Automatic updates are not supported on this platform.".to_owned(),
                    update: None,
                },
                Some(checked_at),
            );
        }

        let updater = match app.updater_builder().build() {
            Ok(updater) => updater,
            Err(error) => {
                return self.set_updater_error(
                    &app,
                    AppUpdatePhaseDto::Checking,
                    &error,
                    Some(mode),
                    None,
                    Some(checked_at),
                );
            }
        };

        match updater.check().await {
            Ok(Some(update)) => self.download_update(&app, update, checked_at, mode).await,
            Ok(None) => self.set_status(
                &app,
                AppUpdateStatusDto::UpToDate { check_mode: mode },
                Some(checked_at),
            ),
            Err(error) => self.set_updater_error(
                &app,
                AppUpdatePhaseDto::Checking,
                &error,
                Some(mode),
                None,
                Some(checked_at),
            ),
        }
    }

    pub async fn install_pending_and_restart(&self, app: AppHandle) -> AppUpdateStateDto {
        let Some(_guard) = self.begin_operation() else {
            return self.state();
        };

        match self
            .inner
            .lock()
            .expect("update state lock poisoned")
            .pending_update
            .take()
        {
            Some(pending_update) => self.install_pending_update(&app, pending_update, true),
            None => self.set_status(
                &app,
                AppUpdateStatusDto::Error {
                    check_mode: None,
                    phase: AppUpdatePhaseDto::Installing,
                    code: AppUpdateErrorCodeDto::NoPendingUpdate,
                    message: "There is no downloaded update ready to install.".to_owned(),
                    update: None,
                },
                None,
            ),
        }
    }

    fn begin_operation(&self) -> Option<BusyGuard> {
        self.busy
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .ok()
            .map(|_| BusyGuard {
                busy: self.busy.clone(),
            })
    }

    async fn download_update(
        &self,
        app: &AppHandle,
        update: Update,
        checked_at: String,
        mode: AppUpdateCheckModeDto,
    ) -> AppUpdateStateDto {
        let metadata = AppUpdateMetadataDto {
            version: update.version.clone(),
            current_version: update.current_version.clone(),
            target: update.target.clone(),
            body: update.body.clone(),
            date: update.date.as_ref().map(ToString::to_string),
        };
        self.set_status(
            app,
            AppUpdateStatusDto::Downloading {
                check_mode: mode,
                update: metadata.clone(),
                downloaded_bytes: 0,
                total_bytes: None,
            },
            Some(checked_at),
        );

        let mut downloaded_bytes = 0_u64;
        let progress_app = app.clone();
        let progress_updates = self.clone();
        let progress_metadata = metadata.clone();

        let bytes = match update
            .download(
                |chunk_length, content_length| {
                    downloaded_bytes = downloaded_bytes.saturating_add(chunk_length as u64);
                    progress_updates.set_status(
                        &progress_app,
                        AppUpdateStatusDto::Downloading {
                            check_mode: mode,
                            update: progress_metadata.clone(),
                            downloaded_bytes,
                            total_bytes: content_length,
                        },
                        None,
                    );
                },
                || {},
            )
            .await
        {
            Ok(bytes) => bytes,
            Err(error) => {
                return self.set_updater_error(
                    app,
                    AppUpdatePhaseDto::Downloading,
                    &error,
                    Some(mode),
                    Some(metadata),
                    None,
                );
            }
        };

        let pending_update = PendingAppUpdate {
            update: Box::new(update),
            bytes,
            metadata: metadata.clone(),
            check_mode: mode,
        };

        if matches!(mode, AppUpdateCheckModeDto::Startup) && !cfg!(target_os = "windows") {
            self.install_pending_update(app, pending_update, false)
        } else {
            self.store_pending_update(pending_update);
            self.set_status(
                app,
                AppUpdateStatusDto::Downloaded {
                    check_mode: mode,
                    update: metadata,
                },
                None,
            )
        }
    }

    fn install_pending_update(
        &self,
        app: &AppHandle,
        pending_update: PendingAppUpdate,
        restart_after_install: bool,
    ) -> AppUpdateStateDto {
        let PendingAppUpdate {
            update,
            bytes,
            metadata,
            check_mode,
        } = pending_update;

        self.set_status(
            app,
            AppUpdateStatusDto::Installing {
                check_mode,
                update: metadata.clone(),
            },
            None,
        );

        match update.install(&bytes) {
            Ok(()) => {
                let state = self.set_status(
                    app,
                    AppUpdateStatusDto::Installed {
                        check_mode,
                        update: metadata,
                    },
                    None,
                );
                if restart_after_install && !cfg!(target_os = "windows") {
                    app.request_restart();
                }
                state
            }
            Err(error) => {
                let state = self.set_updater_error(
                    app,
                    AppUpdatePhaseDto::Installing,
                    &error,
                    Some(check_mode),
                    Some(metadata.clone()),
                    None,
                );
                self.store_pending_update(PendingAppUpdate {
                    update,
                    bytes,
                    metadata,
                    check_mode,
                });
                state
            }
        }
    }

    fn set_status(
        &self,
        app: &AppHandle,
        status: AppUpdateStatusDto,
        checked_at: Option<String>,
    ) -> AppUpdateStateDto {
        let state = {
            let mut inner = self.inner.lock().expect("update state lock poisoned");
            inner.state.revision = inner.state.revision.saturating_add(1);
            inner.state.status = status;
            inner.state.updated_at = timestamp();
            if let Some(checked_at) = checked_at {
                inner.state.checked_at = Some(checked_at);
            }
            inner.state.clone()
        };

        let _ = app.emit(APP_UPDATE_STATE_EVENT, state.clone());
        state
    }

    fn set_updater_error(
        &self,
        app: &AppHandle,
        phase: AppUpdatePhaseDto,
        error: &tauri_plugin_updater::Error,
        check_mode: Option<AppUpdateCheckModeDto>,
        update: Option<AppUpdateMetadataDto>,
        checked_at: Option<String>,
    ) -> AppUpdateStateDto {
        self.set_status(
            app,
            AppUpdateStatusDto::Error {
                check_mode,
                phase,
                code: updater_error_code(error, phase),
                message: error.to_string(),
                update,
            },
            checked_at,
        )
    }

    fn store_pending_update(&self, pending_update: PendingAppUpdate) {
        self.inner
            .lock()
            .expect("update state lock poisoned")
            .pending_update = Some(pending_update);
    }
}

fn updater_error_code(
    error: &tauri_plugin_updater::Error,
    phase: AppUpdatePhaseDto,
) -> AppUpdateErrorCodeDto {
    use tauri_plugin_updater::Error;

    match error {
        Error::EmptyEndpoints | Error::InsecureTransportProtocol => {
            AppUpdateErrorCodeDto::Configuration
        }
        Error::ReleaseNotFound | Error::Reqwest(_) | Error::Network(_) => {
            AppUpdateErrorCodeDto::Network
        }
        Error::Semver(_)
        | Error::Serialization(_)
        | Error::TargetNotFound(_)
        | Error::TargetsNotFound(_)
        | Error::InvalidUpdaterFormat
        | Error::FormatDate
        | Error::UrlParse(_) => AppUpdateErrorCodeDto::Manifest,
        Error::Minisign(_)
        | Error::Base64(_)
        | Error::SignatureUtf8(_)
        | Error::AuthenticationFailed => AppUpdateErrorCodeDto::Signature,
        Error::UnsupportedArch | Error::UnsupportedOs => AppUpdateErrorCodeDto::Unsupported,
        Error::DebInstallFailed
        | Error::PackageInstallFailed
        | Error::FailedToDetermineExtractPath
        | Error::TempDirNotOnSameMountPoint
        | Error::BinaryNotFoundInArchive
        | Error::TempDirNotFound => AppUpdateErrorCodeDto::Install,
        Error::Io(_) | Error::Tauri(_) => match phase {
            AppUpdatePhaseDto::Installing => AppUpdateErrorCodeDto::Install,
            AppUpdatePhaseDto::Checking => AppUpdateErrorCodeDto::Configuration,
            AppUpdatePhaseDto::Downloading => AppUpdateErrorCodeDto::Network,
        },
        _ => AppUpdateErrorCodeDto::Unknown,
    }
}

fn timestamp() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}
