# myrtle_wyckoff

Super Fast Super Secret Super Hot CEX at Myrtle Wyckoff

- [ ] deploy contracts (1 hr)
- [ ] set up settlement orders (cowswap orders) (30 min)
- [ ] set up depositor (1 hr)
- [ ] set up snapshotter (1 hr)

Total: 8 hrs

## Markus

- [x] finish mainnet contract (1 hr)
- [ ] create suave contract (1 hr)

## Nikita

- [x] set up the server (30 min)
- [ ] write to encrypted vol (1 hr)
- [ ] set up dstack dockerized (1 hr)

## Run instructions

```shell
cd myrtle-wyckoff-dstack
docker build -t myrtle-wyckoff-dstack .
docker run -p 8000:8000 myrtle-wyckoff-dstack
```
