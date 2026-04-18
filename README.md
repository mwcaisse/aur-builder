# aur-builder

A wrapper around `aur-utils` to easily create AUR repositories and add/update packages in an automated fashion. All
package building takes place in a Docker container.

## Developing

### Testing

To run the unit tests:

```bash
cargo test --bin aur-builder
```

To run the integration tests:

```bash
cargo test --test '*'
```

To run all tests:

```bash
cargo test
```

The integration tests involve running the Docker container to build a few packages, therefore it takes a while, and is
recommended to run the unit and integration tests separately.