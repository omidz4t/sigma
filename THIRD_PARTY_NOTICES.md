# Third-party licenses

Sigma's own code and documentation are licensed under MIT; see [LICENSE](LICENSE).
This does not relicense its dependencies.

## Chatmail core

Sigma links chatmail core at revision `12e3a2a70a1c0dbca09fc341721ad13d8bbaffe4`,
licensed under the Mozilla Public License 2.0 (MPL-2.0).

- Source: https://github.com/chatmail/core/tree/12e3a2a70a1c0dbca09fc341721ad13d8bbaffe4
- Original license and copyright notice: [chatmail core license](LICENSES/chatmail-core-MPL-2.0.txt)

The included license is reproduced from that upstream revision. It applies to
chatmail core, not to Sigma's independently authored MIT files. Core source can
be obtained from the linked revision; any future modifications to core must
remain available under its applicable license terms.

Other dependencies retain their respective licenses. Cargo.lock records the
resolved dependencies; their license metadata and notices are supplied by their
upstream source packages.

## Heroicons

The landing page uses inline outline icons from [Heroicons](https://heroicons.com), copyright Tailwind Labs, Inc., under MIT. The [license notice](LICENSES/heroicons-MIT.txt) is also embedded in the landing page HTML.

The Delta Chat logo in `landing/static/assets/brands/delta-chat.svg` comes from the official Delta Chat website (https://delta.chat/assets/logos/delta-chat.svg) and identifies the Delta Chat project.

## Vazirmatn

The Persian website font, Vazirmatn Regular, is by Saber Rastikerdar and contributors, distributed under the SIL Open Font License 1.1. Source: https://github.com/rastikerdar/vazirmatn. License included at `landing/static/assets/fonts/OFL.txt`.

## MADMAIL logo

`landing/static/assets/brands/madmail.png` is the unmodified MADMAIL logo from https://github.com/themadorg/madmail/blob/main/landing/src/lib/assets/madmail_logo.png. The upstream repository is distributed under AGPL-3.0; see https://github.com/themadorg/madmail/blob/main/LICENCE. The logo identifies the linked MADMAIL project.

## asciinema player

The AI showcase embeds asciinema-player 3.6.3, copyright Marcin Kulik and contributors, under Apache-2.0. Source: https://github.com/asciinema/asciinema-player. The full license is included in `LICENSES/asciinema-player-Apache-2.0.txt`.

## Store and platform icons

The six SVG marks in `landing/static/assets/stores/` come from Simple Icons 11.15.0, distributed under CC0-1.0. The license is included in `LICENSES/simple-icons-CC0.txt`. Source: https://github.com/simple-icons/simple-icons/tree/11.15.0/icons. Brand marks identify their respective platforms and stores.
