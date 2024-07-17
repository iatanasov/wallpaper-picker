NAME	:=wallpaper-picker
WP_BIN_PATH=${HOME}/.cargo/bin/$(NAME)
SYSTEMD_PATH=${HOME}/.config/systemd/user/${NAME}.service

.DEFAULT_GOAL := help

help: # Print this message
	@grep -h "#" $(MAKEFILE_LIST) | grep -v grep | sed -e 's/\\$$//' | sed -e 's/#//'

install_local: # Local installation
	@cargo install --path ./

install_service: # Install local systemd unit 
	@cat service/wallpaper-picker.service  | sed  's|PATH_TO_BINARY|$(WP_BIN_PATH)|g' > ${SYSTEMD_PATH}
	@systemctl --user daemon-reload 
	@systemctl --user status $(NAME) 
	@systemctl --user restart $(NAME) 
