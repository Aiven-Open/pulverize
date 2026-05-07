# pulverize

A command line tool to break a large file in smaller chunks.

## Overview

Chunks are approximately 4MiB each and selected using the FastCDC2020 algorithm.

The list of chunks is recorded in a JSON manifest.

For each chunk we record its length and a digest.

The digest is computed using the xxh3_128 algorithm.

This tool does not produce the separate chunks on disk, it only generates the manifest.

## License

pulverize is licensed under the Apache license, version 2.0. Full license text is available in the [LICENSE](LICENSE) file.

## Contact

To report any possible vulnerabilities or other serious issues please see our [security](SECURITY.md) policy.
