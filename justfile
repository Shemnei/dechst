comp:
	cargo run -- completions zsh --output-directory {{ justfile_directory() }}
	sudo mv {{ justfile_directory() }}/_dechst /usr/share/zsh/site-functions/

man:
	cargo run -- man --output-directory {{ justfile_directory() }}
	sudo mv {{ justfile_directory() }}/dechst.1 /usr/share/man/man1/

install: comp man
	cargo install --path {{ justfile_directory() }}/crates/dechst-cli
