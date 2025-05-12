# `lnaddrd`: Simple Lightning Address Server

`lnaddrd` is a simple Lightning Address server that allows users to register Lightning Addresses (e.g., `user@lnaddr.org`) which forward incoming payments to the user's LNURL. This makes it easy to provide your own domains to users without also being their LNURL provider.

You can see a demo at https://lnaddr.org.

<div align="center">
  <img src="screenshots/register.png" alt="Registration Form" height="400" style="margin-right: 20px">
  <img src="screenshots/details.png" alt="Address Details" height="400">
</div>

## Features

- **Self-hosted Lightning Address server** written in Rust
- **User registration** for Lightning Addresses
- **Configurable domains** (serve multiple domains)
- **PostgreSQL database** backend
- **Environment variable configuration**
- **Nix/NixOS native deployment**

## Quick Start (with Nix)

### 1. Development Shell

To enter a development shell with all dependencies:

```sh
nix develop
```

This provides Rust, Diesel CLI, PostgreSQL, and other tools.

### 2. Running locally

The dev shell comes with a range of useful `just` commands. Run `just` to see all of them.
When running the server locally make sure you also have a postgres DB running, one can easily be started and initialized as shown below:

```sh
just db-start
just db-create
cargo run -- --domains localhost
```

For the full list of config options see `cargo run -- --help`:

```text
Usage: lnaddrd [OPTIONS]

Options:
      --domains <DOMAINS>...  One or more domain names to serve. Specify multiple times for multiple domains [env: LNADDRD_DOMAINS=]
      --bind <BIND>           The address to bind the server to [env: LNADDRD_BIND=] [default: 127.0.0.1:8080]
      --database <DATABASE>   The database URL [env: LNADDRD_DATABASE_URL=] [default: postgres://localhost:5432/lnaddrd]
      --warning <WARNING>     Warning displayed on registration page [env: LNADDRD_WARNING=]
  -h, --help                  Print help
```

### 3. Running with Docker

You can run `lnaddrd` using Docker. First, build the image (or pull from your registry if available):

```sh
docker build -t lnaddrd .
```

Or pull from GitHub Container Registry (if published):

```sh
docker pull ghcr.io/<your-username-or-org>/lnaddrd:latest
```

Then run the container, passing the required environment variables:

```sh
docker run -p 8080:8080 \
  -e LNADDRD_DOMAINS="yourdomain.com" \
  -e LNADDRD_BIND="0.0.0.0:8080" \
  -e LNADDRD_DATABASE_URL="postgres://user:password@host:5432/lnaddrd" \
  ghcr.io/<your-username-or-org>/lnaddrd:latest
```

- `LNADDRD_DOMAINS`: Comma-separated list of domains to serve (e.g., `lnaddr.org,lnaddr.net`)
- `LNADDRD_BIND`: Address to bind the server to (default: `0.0.0.0:8080` for Docker)
- `LNADDRD_DATABASE_URL`: PostgreSQL connection string
- `LNADDRD_WARNING`: (Optional) Warning message for the registration page

You must have a PostgreSQL database accessible to the container. See the `docker-compose.yml` for an example of running with a local database.

### 4. NixOS Module

`lnaddrd` comes with a NixOS module for easy deployment. Example configuration:

```nix
let
  domains = [ "lnaddr.org" ]
in
{
  services.lnaddrd = {
    enable = true;
    domains = domains;
    database = "postgresql:///lnaddrd?user=lnaddrd";
  };

  services.postgresql = {
    enable = true;
    ensureDatabases = [ "lnaddrd" ];
    ensureUsers = [
      {
        name = "lnaddrd";
        ensureDBOwnership = true;
      }
    ];
  };

  # Example NGINX reverse proxy
  services.nginx = {
    enable = true;
    recommendedTlsSettings = true;
    virtualHosts = builtins.listToAttrs (lib.map (domain: {
      name = domain;
      value = {
        forceSSL = true;
        enableACME = true;
        locations."/" = {
            # Default bind address, can be changed if it collides
            proxyPass = "http://127.0.0.1:8080";
          };
        };
      }) domains)
  };

  networking.firewall.allowedTCPPorts = [ 80 443 ];
}
```

### 5. Environment Variables

- `LNADDRD_DOMAINS`: Comma-separated list of domains to serve (e.g., `lnaddr.org,lnaddr.net`)
- `LNADDRD_BIND`: Address to bind the server to (default: `127.0.0.1:8080`)
- `LNADDRD_DATABASE_URL`: PostgreSQL connection string (default: `postgres://localhost:5432/lnaddrd`)
- `LNADDRD_WARNING`: Optional warning message for the registration page

## Database

`lnaddrd` uses PostgreSQL. Make sure the database and user exist and are accessible by the service.

## Future Feature Ideas

- 402 payment integration
- Letting usrs change LNURLs
- Restrict registration

## License

MIT