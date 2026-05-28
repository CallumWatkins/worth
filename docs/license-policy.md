# License Policy

`license-policy.json` controls which bundled dependency licenses pass `bun run check:licenses` and packaged desktop builds. It is an engineering release gate, not legal advice.

## Fields

- `approvedLicenses` contains SPDX license identifiers that are allowed for any bundled package. Add only licenses that should be accepted globally.
- `approvedExceptions` contains SPDX exception identifiers that are allowed with an approved base license. A license such as `Apache-2.0 WITH LLVM-exception` requires both the base license and exception to be approved.
- `reviewedPackages` contains package-specific approvals for licenses that should not be accepted globally. Use this when the package and license have been reviewed, but future packages using the same license should still require review.
- `licenseOverrides` corrects missing or inaccurate package metadata. Use this when the package's distributed metadata is wrong or incomplete, and there is a reliable source for the actual license.
