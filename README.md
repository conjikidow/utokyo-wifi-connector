# utokyo-wifi-connector

An unofficial command-line tool to manage connections to UTokyo WiFi and eduroam networks using NetworkManager (nmcli).


## Installation

1. Clone this repository:

    ```bash
    $ git clone git@github.com:conjikidow/utokyo-wifi-connector.git
    ```

2. Navigate to the project directory:

    ```bash
    $ cd utwifi
    ```

3. Make the script executable:

    ```bash
    $ chmod +x utwifi
    ```

4. Create a symbolic link to the script in a directory included in your `$PATH`. For example:

    ```bash
    $ sudo ln -s $(pwd)/utwifi /usr/local/bin
    ```

    or

    ```bash
    $ ln -s $(pwd)/utwifi ~/.local/bin
    ```


## Usage

### Add Connections

To add connections to UTokyo WiFi and eduroam networks, run:

```bash
$ utwifi add
```

Follow the prompts to enter your user ID and password.

### Update Connections

To update existing connections, run:

```bash
$ utwifi update
```

Follow the prompts to update the user ID and password for each connection.

### Delete Connections

To delete existing connections, run:

```bash
$ utwifi delete
```

### View Connection Information

To view information about existing connections, run:

```bash
$ utwifi info
```

### Help

For help and available subcommands, run:

```bash
$ utwifi help
```
