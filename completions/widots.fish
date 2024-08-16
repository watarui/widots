# ~/.config/fish/completions/widots.fish

# subcommands
complete -f -c widots -n "__fish_use_subcommand" -a "link" -d "Link dotfiles"
complete -f -c widots -n "__fish_use_subcommand" -a "materialize" -d "Materialize dotfiles"
complete -f -c widots -n "__fish_use_subcommand" -a "load" -d "Apply configuration from TOML file"
complete -f -c widots -n "__fish_use_subcommand" -a "brew" -d "Manage Homebrew"
complete -f -c widots -n "__fish_use_subcommand" -a "deploy" -d "Deploy widots to the local machine"
complete -f -c widots -n "__fish_use_subcommand" -a "fish" -d "Manage Fish shell"
complete -f -c widots -n "__fish_use_subcommand" -a "vscode" -d "Manage Visual Studio Code extensions by using code command"
complete -f -c widots -n "__fish_use_subcommand" -s v -l verbose -d "Output verbose information"
complete -f -c widots -n "__fish_use_subcommand" -s vv -d "Output verbose information more loudly"

# link
complete -c widots -n "__fish_seen_subcommand_from link" -s t -l test -d "Test the dotfiles directory for symlinks and files"
complete -c widots -n "__fish_seen_subcommand_from link" -s f -l force -d "Force create symlinks, overwriting existing files"
complete -c widots -n "__fish_seen_subcommand_from link" -a "(__fish_complete_path)"

# materialize
complete -c widots -n "__fish_seen_subcommand_from materialize" -a "(__fish_complete_path)"

# run
complete -c widots -n "__fish_seen_subcommand_from load" -s t -l test -d "Test the dotfiles directory for symlinks and files"
complete -c widots -n "__fish_seen_subcommand_from load" -s f -l force -d "Force create symlinks, overwriting existing files"
complete -c widots -n "__fish_seen_subcommand_from load" -a "(__fish_complete_path)"

# brew
complete -f -c widots -n "__fish_seen_subcommand_from brew" -a "install" -d "Install Homebrew itself"
complete -f -c widots -n "__fish_seen_subcommand_from brew" -a "import" -d "Import Homebrew packages"
complete -f -c widots -n "__fish_seen_subcommand_from brew" -a "export" -d "Export Homebrew packages"

# fish
complete -f -c widots -n "__fish_seen_subcommand_from fish" -a "install" -d "Install Fish shell"
complete -f -c widots -n "__fish_seen_subcommand_from fish" -a "default" -d "Set Fish as default shell"
complete -f -c widots -n "__fish_seen_subcommand_from fish" -a "fisher" -d "Install Fisher plugin manager"

# vscode
complete -f -c widots -n "__fish_seen_subcommand_from vscode" -a "import" -d "Import VS Code extensions"
complete -f -c widots -n "__fish_seen_subcommand_from vscode" -a "export" -d "Export VS Code extensions"
complete -f -c widots -n "__fish_seen_subcommand_from vscode" -a "code" -d "Ensure code command is available"
