# Backup and restoration

```sh
sigma backup ./backups --passphrase-file ./backup-password
sigma restore ./backups/sigma-chat-backup-EXAMPLE.tar --data-path ./restored --passphrase-file ./backup-password
```

`backup DIRECTORY` invokes core's backup exporter and creates a private output directory on Unix (0700). The backup contains account keys, transport credentials, chats, messages and attachments. Core excludes device-dependent settings and resets device-info messages on import. Core chooses the archive filename. JSON output includes file paths from core's export events. Do not place backups in publicly served or version-controlled folders.

Core encrypts the exported database when `--passphrase-file` is supplied; attachment blobs remain unencrypted TAR entries. The passphrase does not encrypt the entire archive. Without this option, the database is also unencrypted. The file option reads a UTF-8 passphrase, removing one trailing newline (and an optional preceding carriage return); other spaces are preserved. Empty passphrase files are rejected. Use the same passphrase file to restore an encrypted backup. Passphrases are not exposed in command arguments or output. The source password file's permissions remain the user's responsibility.

`restore BACKUP_FILE` requires a new or empty account directory. Existing `account.db` files and nonempty directories are refused before opening a context. Use `--data-path` to choose the destination. Core performs archive and passphrase validation. The original backup is not modified. If import fails, the destination can contain partial state; inspect it and use another empty directory for a retry. The command never erases an existing account to retry automatically.

Stop `listen` before backup or restore. Restored accounts retain the original identity; avoid running the old and restored copies concurrently as separate bots. Verify a restoration with `status`, `chats` and local `read-messages` before starting networking. Keep whole-directory copies with sigma stopped if you also need a raw filesystem snapshot.

Core export does not include sigma’s `apps/` metadata and event journals. To retain saved app names, cursors and captured realtime history, stop the app worker and copy the whole account directory, including `apps/`. After a core-only restore, use `app-bind NAME MESSAGE_ID` to register a restored app message.

Contact aliases are CLI sidecar state (`contact-aliases.tsv`), so core's encrypted backup does not include them. Preserve that private file separately when moving accounts, alongside any other CLI sidecar state you need. Sending checks both the saved contact ID and address before using an alias.
