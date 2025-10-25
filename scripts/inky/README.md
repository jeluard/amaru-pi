Make sure [inky](https://github.com/pimoroni/inky) has been properly setup.
Then in your `amaru-pi` folder, run: `PEER_ADDRESS=192.168.1.61:3001 PATH=$PATH:. ./scripts/badge-per-epoch.sh`

# Requirements

```shell
pip install inky
```

# Usage

```shell
python inky/create_badge.py preview.png 1234 12345 200 100 red

python inky/badge_to_inky.py 1234 12345
```