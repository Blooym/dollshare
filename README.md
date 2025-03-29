# Dollhouse

![Dollware Badge](.assets/88x31.png)

> [!CAUTION]  
> **This project is made for me, my needs, and my infrastructure.**
>
> No support will be offered for this software, and breaking changes to functionalty or features may be made any time.

A safe & encrypted place to share files. 🎀

## Features

- **Ephemeral-first**: Files are treated as temporary and will be automatically deleted based on a configurable time since last access.

- **Storage-efficient**: Files are deduplicated by writing them to disk as `<hash>.<ext>` which helps to minimise storage usage. Hashes are salted with a value generated at first startup which is then stored on disk.

- **Encrypted at rest**: Files are encrypted on-server during upload and a key is attached to the URL sent back to the uploader; No upload can be accessed without the given key, even with access to the backing filesystem. 

- **Configurable and simple to host**: Running the server should be as simple as pulling the docker container or building the binary, changing a few configuration options, and starting the server.

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
| Return HTTPS           | Return all URLs to clients using the `https` scheme. This does not make the internal server use HTTPs. This requires you to run a reverse proxy infront of the server as the request URL hostname will be used when returning URLs to clients.                                                                                                     | `--return-https`           | `DOLLHOUSE_RETURN_HTTPS`           | `false`                       |
| Tokens                 | One or more bearer tokens to use when interacting with authenticated endpoints.                                                                                                                                                                                                                                                                    | `--tokens`                 | `DOLLHOUSE_TOKENS`                 |                               |
| Data path              | A path to the directory where data should be stored. This directory should not be used for anything else as it and all subdirectories will be automatically managed.                                                                                                                                                                               | `--data-path`              | `DOLLHOUSE_DATA_PATH`              | `OS Data Directory/dollhouse` |
| Upload expiry time     | The amount of time since last access before a file is automatically purged from storage.                                                                                                                                                                                                                                                           | `--upload-expiry-time`     | `DOLLHOUSE_UPLOAD_EXPIRY_TIME`     | `31 days`                     |
| Upload expiry interval | The interval to run the expiry check on. This may be an intensive operation if you store thousands of files with long expiry times.                                                                                                                                                                                                                | `--upload-expiry-interval` | `DOLLHOUSE_UPLOAD_EXPIRY_INTERVAL` | `1 hour`                      |
| Upload size limit      | The maximum file size that is allowed to be uploaded.                                                                                                                                                                                                                                                                                              | `--upload-size-limit`      | `DOLLHOUSE_UPLOAD_SIZE_LIMIT`      | `50MB`                        |
| Upload mimetypes       | File mimetypes that are allowed to be uploaded. Supports type wildcards (e.g. 'image/*', '*/*'). MIME types are determined by the magic numbers of uploaded content, if the mimetype cannot be determined the file will be rejected unless all mimetypes are allowed, where it instead will be uploaded as `.unknown` and sent via `octet-stream`. | `--upload-mimetypes`       | `DOLLHOUSE_UPLOAD_MIMETYPES`       | `image/*`, `video/*`          |
