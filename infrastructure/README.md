# Infrastructure

Files in this directory describe how Zilvar is installed and run on my Arch server.

All commands are assumed to be run on the server with current directory being the root of this repository.

## Initial Setup

```sh
# Create a separate user (and group) for the service to run under.
sudo useradd -m zilvar-server
# Create config and runtime directories.
sudo mkdir /var/lib/zilvar /etc/zilvar
# Let the user own the directories.
sudo chown -R zilvar-server:zilvar-server /var/lib/zilvar /etc/zilvar
# Build the server/watchdog binary.
cargo build --release
# "Install" the zilvar binary.
sudo cp target/release/zilvar /usr/bin/
```

### Emailing

```sh
# Generate a key-pair for DKIM signatures
sudo ssh-keygen -t rsa -b 2048 -f /etc/zilvar/dkim_key.private
sudo mv /etc/zilvar/dkim_key.private.pub /etc/zilvar/dkim_key.public
```

Grab the public key and setup a DKIM record for it with `z1` selector.
You'll also need to setup SPF and DMARC records.

### Systemd Service

```sh
# Install the service file.
sudo cp infrastructure/zilvar.service /etc/systemd/system/
# Reload the service files.
sudo systemctl daemon-reload
# Enable the zilvar service and start it immediately.
sudo systemctl enable --now zilvar
```

## Updating

To deploy the latest version of zilvar assuming the setup above was already done:

```sh
cargo build --release && sudo cp --force target/release/zilvar /usr/bin/ && sudo systemctl restart zilvar
```

## Debugging

To see (and follow) latest logs from the zilvar service:

```sh
sudo journalctl -a -f -u zilvar
```
