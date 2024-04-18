# clipat - Remote clipboard server / client

Allows sharing clipboard with remote SSH sessions.

Similar to [lemonade](https://github.com/lemonade-command/lemonade).

## Usage in shell

1. Run server on local machine:
   ```sh
   clipat server
   ```

1. Set up SSH session with remote machine:
   ```sh
   ssh -R 14573:127.0.0.1:14573 user@remote
   ```

1. To copy text on a remote machine:
   ```sh
   cat copy.txt | clipat copy
   ```

1. To paste text on a remote machine:
   ```sh
   clipat paste > paste.txt
   ```

## Usage with neovim

This will replace your + and * registers with clipat.

```lua
vim.g.clipboard = {
  name = 'clipat',
  copy = {
    ['+'] = { 'clipat', 'copy' },
    ['*'] = { 'clipat', 'copy' },
  },
  paste = {
    ['+'] = { 'clipat', 'paste' },
    ['*'] = { 'clipat', 'paste' },
  },
}
```

## Security considerations

By default, clipat only listens on 127.0.0.1. If you want to listen on all
interfaces, you can use the `--listen` option. This is not recommended, as it
allows anyone on the network to access your clipboard.

It is also not recommended to use clipat on multi-user systems, as any user
could potentially access your clipboard.

Communications between the client and server are not encrypted. However, if you
pipe the connection through SSH, this will take care of encryption over internet.


## License

clipat is licensed under the MIT license. Full license text is available in the
[LICENSE](LICENSE) file.
