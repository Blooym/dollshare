# Dollhouse

![Dollware Badge](.assets/88x31.png)

> [!CAUTION]  
> **This project is made for me, my needs, and my infrastructure.**
>
> No support will be offered for this software. Breaking changes to functionalty or features may be made any time.

A safe & encrypted place to share files. 🎀

## Features

- **Ephemeral-first**: All uploads are treated as temporary and will be automatically deleted based on time since last access.

- **Storage-efficient**: All uploads are deduplicated by writing them to disk as `<hash>.<ext>`. Hashes are salted with a persistent key generated on first-time startup.

- **Encrypted at rest**: All uploads are encrypted by the server during upload and the decryption key is sent to the uploader and is not stored by the server, No upload can be accessed without the deceyption key, even with access to the filesystem.
  - Note: as encyption and decryption is handled server-side a malicious actor with access to the server could intercept data unencrypted or log decryption keys. This is a considered flaw thar allows embedding uploads on any website.

- **Configurable and simple**: Running the server should be as simple as pulling the docker container or building the binary, changing a few configuration options, and starting the server.

## Setup

### Docker

1. Copy [compose.yml](./compose.yml) to a local file named `compose.yml` or add the
   service to your existing stack and fill in the environment variables.
   Information about configuration options can be found in the
   [configuration](#configuration) section.

2. Start the stack

```
docker compose up -d
```

### Manual

1. Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed and
   in your `$PATH`.
2. Install the project binary

```
cargo install --git https://github.com/Blooym/dollhouse.git
```

3. Set configuration values as necessary.
   Information about configuration options can be found in the
   [configuration](#configuration) section.

```
dollhouse
```

## Configuration

Dollhouse is configured via command-line flags or environment variables and has full support for loading from `.env` files. Below is a list of all supported configuration options. You can also run `dollhouse --help` to get an up-to-date including default values.

| Name                   | Description                                                                                                                                                                                                                                                                                                                                        | Flag                       | Env                                | Default                       |
| ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------- | ---------------------------------- | ----------------------------- |
| Address                | The internet socket address that the server should be ran on.                                                                                                                                                                                                                                                                                      | `--address`                | `DOLLHOUSE_ADDRESS`                | `127.0.0.1:8731`              |
| Public URL             | The base url to use when generating links to uploads.                                                                                                                                                                                                                                                                                              | `--public-url`             | `DOLLHOUSE_PUBLIC_URL`             | `http://127.0.0.1:8731`       |
| Tokens                 | One or more bearer tokens to use when interacting with authenticated endpoints.                                                                                                                                                                                                                                                                    | `--tokens`                 | `DOLLHOUSE_TOKENS`                 |                               |
| Data path              | A path to the directory where data should be stored. This directory should not be used for anything else as it and all subdirectories will be automatically managed.                                                                                                                                                                               | `--data-path`              | `DOLLHOUSE_DATA_PATH`              | `OS Data Directory/dollhouse` |
| Upload expiry time     | The amount of time since last access before a file is automatically purged from storage.                                                                                                                                                                                                                                                           | `--upload-expiry-time`     | `DOLLHOUSE_UPLOAD_EXPIRY_TIME`     | `31 days`                     |
| Upload expiry interval | The interval to run the expiry check on. This may be an intensive operation if you store thousands of files with long expiry times.                                                                                                                                                                                                                | `--upload-expiry-interval` | `DOLLHOUSE_UPLOAD_EXPIRY_INTERVAL` | `1 hour`                      |
| Upload size limit      | The maximum file size that is allowed to be uploaded.                                                                                                                                                                                                                                                                                              | `--upload-size-limit`      | `DOLLHOUSE_UPLOAD_SIZE_LIMIT`      | `50MB`                        |
| Upload mimetypes       | File mimetypes that are allowed to be uploaded. Supports type wildcards (e.g. 'image/*', '*/*'). MIME types are determined by the magic numbers of uploaded content, if the mimetype cannot be determined the file will be rejected unless all mimetypes are allowed, where it instead will be uploaded as `.unknown` and sent via `octet-stream`. | `--upload-mimetypes`       | `DOLLHOUSE_UPLOAD_MIMETYPES`       | `image/*`, `video/*`          |
