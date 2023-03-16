# Oreally - An O'Reilly book downloader CLI

## Description

`oreally` is a command-line interface (CLI) tool for downloading O'Reilly books. It provides a simple and easy-to-use interface for downloading and managing your O'Reilly books.

## Dependencies

`oreally` uses `docker` and `sqlite` underneath to do the magic.

```shell
brew install docker
brew install sqlite
```

## Configuration

To avoid having to pass the options `--auth` and `--folder` you can set them in your `.bashrc`.

```bash
export OREALLY_AUTH="[email]:[password]" # e.g: you@example.com:my_super_secret_pass
export OREALLY_FOLDER="~/Books" # Folder were the books will be saved
```

To queue books you will need to run:

```shell
oreally init
```

This CLI can be used to download books individually, e.g:

```shell
oreally download --auth [email:password] --folder ~/Books --url https://learning.oreilly.com/library/view/book-title/123456789/
```

## Usage

Usage:
  `oreally <command> [options]`

Commands:

* `download <book-url>`: Download a book from the given O'Reilly book URL.
* `queue <book-url>`: Queue a book for downloading from the given O'Reilly book URL.
* `start`: Start processing books queue.
* `init`: Initialize configuration for oreally.
* `list`: List all queued and downloaded books.
* `help`: Print help information.

Options:

* `-h, --help`: Print help information.

## Commands

### download

Download the O'Reilly book from the given URL. The book will be downloaded and saved in the default download location specified in the configuration file.

```txt
$ oreally download --help

Usage: oreally download [OPTIONS] --url <URL>

Options:
      --url <URL>
      --auth <AUTH>
      --folder <FOLDER>
  -h, --help             Print help information
```

Example:

```shell
oreally download --url https://learning.oreilly.com/library/view/book-title/123456789/
```

### queue

Queue the O'Reilly book from the given URL for downloading. The book will be added to the queue and will be downloaded when the `start` command is executed.

```txt
$ oreally queue --help

Usage: oreally queue --url <URL>

Options:
      --url <URL>
  -h, --help       Print help information
```

Example:

```shell
oreally queue --url https://learning.oreilly.com/library/view/book-title/123456789/
```


### start

Start downloading books from the queue. The books in the queue will be downloaded one by one in the order they were added.

```shell
$ oreally start --help

Usage: oreally start [OPTIONS]

Options:
      --auth <AUTH>
      --folder <FOLDER>
  -h, --help             Print help information
```

Example:

```shell
oreally start
```

### init

Initialize configuration for oreally. This command sets up the default download location and other settings.

Example:

```shell
oreally init
```

### list

List all queued and downloaded books. This command provides a list of all the books that are currently queued or have been downloaded.

Example:

```shell
oreally list
```

### help

Print help information for the `oreally` CLI.

Example:

```shell
oreally --help
```

## Contributing

Bug reports and pull requests are welcome on GitHub at <https://github.com/arturictus/oreally-rs>. This project is intended to be a safe, welcoming space for collaboration, and contributors are expected to adhere to the [code of conduct](https://github.com/arturictus/oreally-rs/blob/master/CODE_OF_CONDUCT.md).

## License

The gem is available as open source under the terms of the [MIT License](https://opensource.org/licenses/MIT).

## Code of Conduct

Everyone interacting in the Oreally project's codebases, issue trackers, chat rooms and mailing lists is expected to follow the [code of conduct](https://github.com/arturictus/oreally-rs/blob/master/CODE_OF_CONDUCT.md).
