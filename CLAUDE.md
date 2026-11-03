# CLAUDE.md

Règles durables pour travailler sur Tegami Mail. Ce qui a été fait, les
décisions et la suite sont dans [docs/SUIVI.md](docs/SUIVI.md) : le lire en
début de session, le tenir à jour après chaque étape poussée.

## Le projet

Client de messagerie GNOME (Rust, GTK 4, libadwaita, relm4, WebKitGTK 6),
issu de Hylki 1.46 (AGPL-3.0-or-later). Le propriétaire écrit en français ;
répondre en français.

Le redesign suit [docs/DESIGN.md](docs/DESIGN.md) et
[docs/DESIGN_STRINGS.md](docs/DESIGN_STRINGS.md), avec les maquettes dans
`data/repo/design/` (01 à 17). Les lire avant de toucher l'interface.

## Règles de code

- Chaînes visibles en anglais dans le code, toujours via `src/i18n.rs` :
  `i18n`, `i18n_f` (paramètres nommés `{name}`), `ni18n` / `ni18n_f` pour
  les pluriels, `i18n_noop` pour les tables. Jamais de phrase construite
  par concaténation.
- Après un ajout ou un changement de chaîne : `tools/update-pot.sh` (dans
  le conteneur), puis traduire en français dans `po/fr.po` avec
  `tools/dev/set-po.py po/fr.po 'msgid' 'traduction' …` (pluriel :
  `'sing|plur'`), puis `msgfmt --check po/fr.po -o /dev/null`. Le français
  du catalogue utilise une espace normale avant « : » et l'apostrophe ’.
- Libellés en anglais en casse de phrase : majuscule au premier mot
  seulement (« Mark all as read »), sauf noms propres et produits (GNOME
  Online Accounts, OpenPGP, Tegami Mail). Décision du propriétaire,
  2026-11-03.
- Commentaires : le pourquoi, pas le quoi. Chaque module commence par un
  `//!` qui dit ce qu'il possède. Une fonction au comportement non évident
  a un `///` avec la raison des choix. Citer le numéro d'issue pour un
  contournement. Un test à côté de chaque correction ou logique pure.
- Libadwaita d'abord : ses widgets, ses couleurs nommées (`@accent_bg_color`,
  `@card_bg_color`…), ses classes. CSS maison minimal (`src/styles.css`,
  et `src/app/theme.rs` pour ce qui dépend du thème, priorité supérieure).
  Le CSS de GTK n'a ni `!important` ni marges logiques (`margin-end`) :
  utiliser `margin-left/right` + variante `:dir(rtl)`.
- Accessibilité : texte secondaire à 70 % d'opacité minimum (contraste
  4.5:1) ; jamais d'information par la couleur seule ; rien de caché
  derrière le survol ; libellé accessible sur chaque bouton icône.
- Une action à un seul endroit. Ne supprimer une fonctionnalité existante
  qu'avec l'accord du propriétaire : demander.
- Un changement de valeur par défaut d'un réglage se fait dans la struct
  (`src/config/privacy.rs`…) et dans les fichiers de référence
  `src/config/testdata/*.toml` ; les réglages déjà enregistrés gardent
  leur valeur.

## Git et CI

- Le propriétaire pousse directement sur `main` ; c'est convenu. Un commit
  par étape cohérente, message en anglais : un titre, puis le quoi et le
  pourquoi, se référer aux sections de DESIGN.md. Terminer par les lignes
  d'attribution demandées par l'environnement.
- `cargo fmt --check` avant chaque commit (rustfmt tourne sur l'hôte).
- La CI (`.github/workflows/ci.yml`) compile et teste sur Debian 13 et
  construit le snap ; les journaux sont publiés sur les branches
  `ci-logs` et `ci-logs-snap` (`git fetch --depth=1 origin ci-logs`).
  Suivre un run : `gh run watch <id> --repo DocteurScience/Tegami_Mail`.
  Un échec du snap à l'étape `gnome/cleanup` a déjà été passager.

## Compiler, tester, capturer

L'hôte (Ubuntu 24.04) n'a pas libadwaita ≥ 1.7. On compile dans un
conteneur Ubuntu 26.04 (`deb.debian.org` est bloqué par le réseau des
sessions cloud) :

- `tools/dev/container-setup.sh` depuis la racine : crée ou relance le
  conteneur `tgu` (dépôt monté sur `/src`, cible `target-ubu/`, exclue de
  git via `.git/info/exclude`). Une première compilation prend ~10 min.
- Tests : `docker exec -w /src -e CARGO_TARGET_DIR=/src/target-ubu tgu bash -c 'xvfb-run -a cargo test --locked'`
- Vérification rapide : même commande avec `cargo check --locked`.
- Capture du mode démo : `docker exec tgu bash /run.sh NOM [VAR=val…]`
  puis `docker cp tgu:/shots/NOM.png …`. Options dans `tools/dev/shot.sh`
  (DELAY, PRIVACY, WINDOW) et les crochets `TEGAMI_SHOWCASE_*` du code
  (`src/app/showcase*`, `TEGAMI_SHOWCASE_REPLY=3 TEGAMI_SHOWCASE_TOP=1`
  pour le composeur, `TEGAMI_SHOWCASE_ATTACH=…`, `TEGAMI_SHOWCASE_TAGS=panel|new`).
  Les captures sont en 2x.
- `WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS=1` n'est utilisé que pour ces
  captures locales (le propriétaire l'a accepté) ; jamais dans le code.
- Instantanés HTML du lecteur : `TEGAMI_BLESS_GOLDEN=1` régénère
  `src/ui/message_view/testdata/*.html` ; relire le diff avant de commiter.
- Un `Edit` après `cargo fmt` échoue souvent : relire la zone d'abord.
- Si Docker ne répond plus (session endormie) : relancer
  `tools/dev/container-setup.sh`.

## Repères d'architecture

- `src/app.rs` + `src/app/*` : la fenêtre principale (composant relm4
  `AppModel`, messages `AppMsg` dans `msg.rs`, routage dans `router.rs`).
  Barre latérale `src/ui/sidebar/`, liste `src/ui/message_list/` et
  `src/ui/message_row/`, lecteur `src/ui/message_view/` (page WebKit
  construite dans `document.rs`, CSS `wrapper_css.rs`, script
  `size_script.rs`, pont page → Rust `bridge.rs`), composeur
  `src/ui/compose/` (éditeur `src/ui/rich_editor/`, scripts JS inclus ;
  certains sont « épinglés » par empreinte dans les tests).
- Réglages : `src/config/` (privacy.toml pour les préférences, toolbar.toml,
  window.toml, sidebar.toml) ; fenêtre Réglages `src/ui/preferences/`,
  comptes/filtres/étiquettes `src/ui/accounts/`.
- Menus contextuels maison : `src/ui/context_menu.rs` (`MenuEntry`, avec
  `.destructive()`, `.checked()`, `.selected()`).
