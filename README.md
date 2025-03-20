# Dollhouse

![Dollware Badge](.assets/88x31.png)

> [!CAUTION]  
> **This project is made for me, my needs, and my infrastructure.**
>
> No support will be offered for this software, and breaking changes to functionalty or features may be made any time.

A safe, encrypted & privacy-focused place to share files 🎀🏠

## Features

- **Ephemeral-first**: Files are treated as temporary and will be automatically deleted based on a configurable time since last access. The server is for file sharing, not for file storage.

- **Storage-efficient**: Files are deduplicated on a by writing them to disk as `<hash>.<ext>` which helps to minimise storage usage.

- **Encrypted at rest**: Files are encrypted on upload via a fully randomized encryption key generated on upload and attached to the URL sent back to the uploader; No upload can be accessed without the given key, even with access to the backing filesystem. 
  * Note: due to files being named by unencrypted hash the content can be inferred if the hash is known. I currently think this is fine. 

- **Multi-user**: Authorization tokens can be provided by the server operator to trusted individuals to allow them to upload their own content privately.
   * Note: due to files being named by unencrypted hash the encryption key of a file could be rotated by another user on accident. This will eventually be fixed by appending a user ID to file names and doing deduplication on a per-user level.

- **Configurable and simple to host**: Running the server be as pulling the docker container or building the binary, changing a few configuration options, and starting the server.

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

3. Copy `.env.example` to `.env` and fill in the values as necessary.
   Information about configuration options can be found in the
   [configuration](#configuration) section.

4. Run the project from the same directory as `.env`

```
dollhouse
```

## Configuration

Dollhouse is configured via command-line flags or environment variables and has full support for loading from `.env` files. Below is a list of all supported configuration options. You can also run `dollhouse --help` to get an up-to-date including default values.

| Name                | Description                                                                                                                                                                                                                | Flag               | Env                        |
| ------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------ | -------------------------- |
| Address             | The socket address that the local server should be hosted on                                                                                                                                                               | `--address`        | `DOLLHOUSE_ADDRESS`        |
| Public URL          | The public url that this server will be exposed as to the internet. This only impacts what base is used when links are sent to users.                                                                                      | `--public-url`     | `DOLLHOUSE_PUBLIC_URL`     |
| Token               | One or more bearer tokens to use when interacting with authenticated endpoints.                                                                                                                                            | `--tokens`         | `DOLLHOUSE_TOKENS`         |
| Upload expiry time  | The amount of time since last access that can elapse before a file is automatically purged from storage.                                                                                                                   | `--expiry-time`    | `DOLLHOUSE_EXPIRY_TIME`    |
| Upload storage path | Where all uploads should be stored locally. This directory should ONLY contain uploads as it is automatically purged and exposed to the internet.                                                                          | `--uploads-path`   | `DOLLHOUSE_UPLOADS_PATH`   |
| Upload limit        | The maximum size of file that can be uploaded.                                                                                                                                                                             | `--upload-limit`   | `DOLLHOUSE_UPLOAD_LIMIT`   |
| Limit to media      | Whether to enforce uploads be of either the `image/*` or `video/*` MIME type. MIME types are determined by the magic numbers of uploaded content, this process is not perfect but will fail-closed on unknown media types. | `--limit-to-media` | `DOLLHOUSE_LIMIT_TO_MEDIA` |
