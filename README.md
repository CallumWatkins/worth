<p align="center">
    <img width="150" src="./public/logo.png" alt="logo">
</p>
<h1 align="center">Worth</h1>
<p align="center">
A balance tracking desktop app, made with <a href="https://v2.tauri.app">Tauri 2</a>, <a href="https://nuxt.com">Nuxt 4</a>, and <a href="https://ui.nuxt.com/">Nuxt UI 4</a>.
</p>

## Technologies

- Nuxt v4
- Tauri v2
- NuxtUI v4
- TailwindCSS v4
- Typescript
- ESLint

## Prerequisites

- Install [Tauri prerequisites](https://tauri.app/start/prerequisites).
- Install [bun](https://bun.sh).

## Dev
Start the app for development.

```sh
# install dependencies
$ bun install

# start the project
$ bun run tauri:dev
```

## Build
Generate the Nuxt static output and bundle the project under `src-tauri/target`.

```sh
$ bun run tauri:build
```

## Debug
Generate the Nuxt static output with the ability to open the console and bundle the project under `src-tauri/target`.

```sh
$ bun run tauri:build:debug
```

## Bump version number
Use the `bumpp` interactive CLI to bump version numbers

```sh
$ bun run bump
```
