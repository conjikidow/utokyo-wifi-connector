# utokyo-wifi-connector

An unofficial command-line tool to manage connections to UTokyo WiFi and eduroam networks using NetworkManager (nmcli).


## Installation

1. Clone this repository:

    ```bash
    git clone git@github.com:conjikidow/utokyo-wifi-connector.git
    ```

2. Navigate to the project directory:

    ```bash
    cd utwifi
    ```

3. Make the script executable:

    ```bash
    chmod +x utwifi
    ```

4. Create a symbolic link to the script in a directory included in your `$PATH`. For example:

    ```bash
    sudo ln -s $(pwd)/utwifi /usr/local/bin
    ```

    or

    ```bash
    # Make sure that ~/.local/bin is included in your $PATH
    ln -s $(pwd)/utwifi ~/.local/bin
    ```

5. [Optional] (If you are using `zsh`) Generate the `zsh` completion script and add it to your `fpath`. For example:

    ```zsh
    # Make sure that ~/.zfunc is included in your $fpath
    utwifi completion > ~/.zfunc/_utwifi
    ```

    In order to add `~/.zfunc` to your `fpath` and enable completion, add the following lines to your `.zshrc`:

    ```zsh
    export fpath=(~/.zfunc $fpath)
    autoload -U compinit && compinit
    ```


## Usage

### Add Connections

To add connections to UTokyo WiFi and eduroam networks, run:

```bash
utwifi add
```

Follow the prompts to enter your user ID and password.

### Update Connections

To update existing connections, run:

```bash
utwifi update
```

Follow the prompts to update the user ID and password for each connection.

### Delete Connections

To delete existing connections, run:

```bash
utwifi delete
```

### View Connection Information

To view information about existing connections, run:

```bash
utwifi info
```

### Generate Completion Script

To generate a completion script for `zsh`, run:

```bash
utwifi completion
```

Currently, only `zsh` is supported.

### Help

For help and available subcommands, run:

```bash
utwifi help
```
