if [ "${USER:-}" = "root" ] \
    && [ -z "${SSH_CONNECTION:-}" ] \
    && [ -t 0 ] \
    && [ "$(/usr/bin/tty)" = "/dev/tty1" ] \
    && [ -x /usr/local/bin/amaru-pi-startup.sh ]; then
    exec /usr/local/bin/amaru-pi-startup.sh
fi