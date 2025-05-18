# Cyklobazar.cz Watch Doggo

Malá webová appka, která mně (nebo komukoli jinýmu) dovoluje dostávat emailové notifikace na nové inzeráty na [cyklobazaru](https://www.cyklobazar.cz/) - zdarma.

**Zatím stále ve výstavbě.**

## Plánované funkce

- libovolné filtrování inzerátů, tak jako přímo na Cyklobazaru
- volitelná frekvence notifikací - hodina, den, týden
- upozornění na nové inzeráty přímo do mailu
- neomezený počet hlídacích psů

## TODO

- [x] setup network at the server laptop
- [ ] setup email sending
  - [x] configure SPF (i.e. allow zilvar public IP)
  - [x] create DKIM keys
  - [x] configure DKIM (add public key DNS record), set policy to reject
  - [x] send emails with https://github.com/gzbakku/mail-send
- [x] send new ads via email instead of printing them out
- [x] persist data over runs
- [ ] create service file, install as a systemd service
- [ ] implement web interface
  - [ ] homepage / create doggo
  - [ ] confirm new doggo
  - [ ] kill doggo
- [ ] use logging lib
