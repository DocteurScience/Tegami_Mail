# Suivi du redesign

Journal de ce qui est fait, des décisions et de ce qui reste, pour
reprendre le travail dans une autre conversation. Les règles durables
(conventions, commandes, architecture) sont dans [CLAUDE.md](../CLAUDE.md).
Mettre ce fichier à jour à chaque étape poussée.

Dernière mise à jour : 2026-11-02, après `40a6c7e`.

## État

- Base : import de Hylki 1.46.0, renommé Tegami Mail (commits du
  2026-10-31 et du 2026-11-01 jusqu'à `fd0ccce`).
- CI : verte sur tous les commits jusqu'à `deef37d` (build, tests, snap) ;
  les suivants à vérifier (`gh run list --repo DocteurScience/Tegami_Mail`).
- Tests : 1156 réussis, 40 ignorés.
- Référence du design : `docs/DESIGN.md`, `docs/DESIGN_STRINGS.md`,
  maquettes `data/repo/design/01` à `17`.

## Fait

### Remise en état (2026-11-01)

| Commit | Contenu |
| --- | --- |
| `e79c0b8` | 7 tests cassés par le renommage (encodage `%20`, empreintes des scripts), test de report faux le dimanche soir, `librsvg2-common` en CI |
| `93c4980` | `cargo fmt` sur les lignes allongées par le renommage |
| `1c53a38` | CI : `actions/checkout` et `upload-artifact` en v7 (Node 24) |
| `343b9df` | Catalogue de traduction régénéré (il ne l'avait pas été depuis le renommage) |

### Liste (maquettes 01, 08)

| Commit | Contenu |
| --- | --- |
| `87156bb` | Cartes (fond carte, bordure, 8 px entre), sélection = contour d'accent 2 px, texte secondaire à 70 %, contraste élevé, perte de focus (#274) en contour gris, curseur j/k en pointillés |
| `f156fac` | Étiquettes sous l'aperçu, 2 au plus puis « +n » (noms dans l'infobulle), puces teintées du tableau de DESIGN.md (test de contraste) |
| `3b032be` | En-tête : Nouveau message au début (clic droit : modèle), nom de la vue au centre, recherche à la fin ; sélecteur Tous / Non lus / Favoris (AdwToggleGroup) ; ces deux filtres quittent le menu de tri |

### Conversation (maquette 02)

| Commit | Contenu |
| --- | --- |
| `17b65ed` | « 6 messages · 4 participants » sous l'objet ; messages lus repliés par défaut, carte repliée sur une ligne (avatar, expéditeur, premiers mots, date) |

### Composeur (maquettes 04, 05, 11, 12, 13)

| Commit | Contenu |
| --- | --- |
| `9949244` | Fenêtre par défaut ; en-tête Annuler / titre (« Répondre à … », « Brouillon enregistré à … ») / ⋯ / Envoyer ; menu ⋯ (brouillon, modèle, cloud, contacts, OpenPGP, accusé de réception, fenêtre, supprimer en rouge) ; `MenuEntry::destructive()` |
| `be530af` | Section « Historique de la conversation » (compte, Masquer/Afficher, « Afficher le message précédent » au-delà de 2 niveaux), affichée par défaut ; `fold.js` réécrit |
| `fd745ea` | Composeur à plat en fenêtre (cartes gardées en ligne), bouton « Cc / Cci », fond gris de l'historique |
| `140d041` | Libellés des champs à gauche (SizeGroup, mode « side label » de `RecipientRow`), « Joindre un fichier… » après la mise en forme (`ToolFlow::append_kept`) |
| `c7990ff` | Pièce jointe oubliée (`compose/forgotten.rs`, liste de mots traduisible) ; refus SMTP 552 / 5.3.4 expliqué |
| `5be32a8` | Pièces jointes sous le texte, « 3 fichiers · 18,4 Mo », « TYPE · taille », avertissement ≥ 10 Mo et « Envoyer un lien » |
| `bd2b034` | Zones de dépôt : titres du design, bordure accent pleine force |

### Pièces jointes reçues (maquette 10)

| Commit | Contenu |
| --- | --- |
| `9947dc8` | Section sous le message (compte, total, Tout enregistrer), cartes avec aperçu (image ou 1re page PDF, `thumbnails::card_preview`), avertissement exécutable, menu ⋯ (Ouvrir, Ouvrir avec…, Enregistrer sous…, Tout enregistrer…) ; verbe `AttVerb` ; tiroir désactivé par défaut |

### Étiquettes (maquettes 07, 08, 09)

| Commit | Contenu |
| --- | --- |
| `a76fa13` | Mot-clé sans accents (« Réunion » → `Reunion`) |
| `abb5741` | Panneau Étiquettes (`ui/tags_panel.rs`) ; boîte Nouvelle étiquette (`ui/tag_dialog.rs`) ; mot-clé unique ; Gérer → Réglages, page Étiquettes |
| `a22d668` | « + » sur la ligne Étiquettes de la barre latérale |

### Menus et en-tête du lecteur (maquette 06)

| Commit | Contenu |
| --- | --- |
| `a019b3c` | Barre du lecteur par défaut : Répondre (avec libellé), Répondre à tous, Transférer, Archiver, Supprimer … Étiquettes, ⋯ ; Étiquettes reste visible quand le groupe droit est replié ; « Supprimer le dossier… » en rouge |
| `3e900c1` | Menu de tri : section Afficher (conversations, aperçu, avatars) liée aux réglages |
| `636b0d7` | CLAUDE.md, ce journal, scripts `tools/dev/` |
| `fd7a789` | Menu principal : Synchroniser maintenant (F5), Ajouter un compte…, Comptes en ligne GNOME…, Aide ; retrait d'Annuler, Volet de lecture, Mode Focus, Plein écran (les raccourcis restent) |
| `583c960` | Composeur dans le volet de lecture : l'historique ne montre que le dernier message reçu (`history_levels` 1 ; 2 en fenêtre) |
| `7d5bc0a` | Menu du clic droit d'une ligne réduit à la maquette |
| `89e8aa0` | « Copier vers… » supprimé partout (menus liste, sélection, lecteur) avec son code ; docs et CHANGELOG mis à jour |
| `15f9b65` | Interrupteur clair/sombre par message supprimé (en-tête du lecteur, réglage, clé `theme_switch_shown`) ; Réglages → Lecture → Apparence des messages reste |
| `3713aeb` | En-tête de la barre latérale : titre « Courrier », plus d'Actualiser au début (le rail et le panneau étroit gardent le leur) |
| `f09700d` | Composeur : Envoyer en un seul bouton (icône + libellé), « Envoyer plus tard » retiré (l'Outbox envoie encore ce qu'une version antérieure a programmé), plus de bouton Fermer dans la fenêtre du composeur |
| `f504a8b` | Menu ⋯ du composeur : section Format (`MenuEntry::heading`), Vérifier l'orthographe, Langue du correcteur (`spelling_choices` partagé avec le clic droit) ; bouton de format retiré de la barre |
| `d1c6780` | Menu ⋯ du composeur réduit à la maquette : retrait d'Enregistrer comme modèle, Envoyer vers le cloud (fichiers choisis), Ouvrir les contacts, Ouvrir dans une fenêtre / Replier, avec leur code (bascule fenêtre ↔ volet, sélecteur de contacts) |
| `1e9d453` | Markdown et Source HTML supprimés : `ComposeFormat` = Rich / Plain (alias serde « markdown »/« html » → Rich), mode source et Aperçu du composeur, moteur Markdown et `pulldown-cmark` ; le mode source reste pour la signature HTML dans les Réglages |
| `a357bad` | Champ De en bouton (avatar, nom, adresse) ouvrant « Envoyer depuis » + « Gérer les comptes… » ; crayon (#347) retiré, un From hors identités venu du préremplissage (+tag, brouillon) est gardé et affiché ; `MenuEntry::subtitle`, classe `.secondary-text` (70 %) |
| `d90278f` | Navigateur de pièces jointes supprimé : page galerie, ligne de la barre latérale, réglage, état enregistré, requêtes du cache, scan BODYSTRUCTURE. Gardés : `AppMsg::OpenMessageAt` (liens `tegami:`), aides « Suppression… » du tiroir, table `attachment_meta` |
| `8e38bf6` | Vue Contacts supprimée : page, ligne et arbre de la barre latérale, pied de barre, raccourci g c, réglages d'emplacement et de recherche, code d'édition des contacts. Gardés : carnets et Réglages → Contacts (import/export/tout supprimer), « Ajouter aux contacts », suggestions et groupes du composeur, noms et photos |
| `40a6c7e` | Section Filtres de la barre latérale supprimée (ligne unifiée, vue fusionnée, réglage d'emplacement, repli du rail). Gardés : règles de filtre, marque sur le dossier où une règle range, « Appliquer les filtres » |
| `bb5953c` | Vue Lecteur supprimée : interrupteur de l'en-tête, réglages (Lecture), partie « Vue Lecteur » du mode Focus, bascule de la fenêtre de message, clés `reader_mode` / `reader_switch` / `reader_default`. La feuille `crate::mail::reader` reste : elle affiche les traductions de messages en texte brut. Paramètre `reader` de `conversation_document` retiré ; instantané `three_cards.html` régénéré |
| `13ac3a4` | Tables `attachment_meta`/`attachment_scan` supprimées (schéma v22, migration `DROP`), leurs écritures (workers, `strip.rs`, démo) et le préchargement des dossiers « galerie » ; le préchargement des pièces jointes récentes reste |
| `80e9d01` | Restes du mode Concentration : boutons des en-têtes et avatars de la liste sortis de leurs révélateurs, mesure du repli simplifiée (plus d'attente d'ouverture, remesure quand un bouton apparaît), `FOCUS_ANIM_MS` et CSS `.focus-fade` retirés ; vérifié par captures à 1430, 1090 et 820 px |
| `d0c271f` | Relecture de la doc : Undo Send (plus de réponse dans le volet), README des docs et deux phrases du CHANGELOG (Markdown/HTML, réglage du composeur dans le volet) ; FEATURES, DOCUMENTATION et KEYBOARD_SHORTCUTS déjà à jour par ailleurs |
| `29259e7` | Outillage de comparaison aux maquettes : `tools/dev/grab.sh`, `popgrab.sh`, `mockup-shots.sh`, `mockup-crop.py` ; Adwaita Sans et outils X dans le conteneur, proxy apt rafraîchi à chaque démarrage. Série de 17 captures envoyée au propriétaire (2026-11-03) pour une analyse de design |
| `83df670`, `5631585`, `ce58a5c`, `6a7d4cb`, `39b63ef` | Audit du code (2026-11-03) : clippy `--all-targets` à zéro ; entrées obsolètes des catalogues purgées (`update-pot.sh` les retire désormais) ; 27 règles CSS et 5 icônes mortes retirées ; actions rapides du tiroir visibles au focus clavier ; Ctrl+Maj+A mort et relais `ZoomKey`/`ComposerKey` retirés ; ~40 commentaires périmés (All Inboxes, réponse dans le volet, galerie) ; panique IMAP corrigée (lecture sans session après un envoi de brouillon). Restent, à décider : la mécanique des vues unifiées ne sert plus qu'aux Favoris (simplification possible) ; la touche `d` double `c` |
| `2b3839e` … `6d1b973` | Retour de design du 2026-11-04 (Claude design). P1 : compte en texte neutre seulement si la vue mêle des comptes, étiquettes de la conversation sous le titre ; grand texte (le lecteur suivait déjà, la capture était fausse ; lignes repliées sur une ligne) ; contraste élevé par fournisseur CSS suivant libadwaita ; compteurs de la barre latérale en texte ; menus sans icônes, coche pour les choix. P2 : la barre agit sur le dernier message, contour seulement pour une carte choisie, chevrons ; ligne « À : moi · 9 oct., 06:53 » ; icônes B/I/U/S et lien, boutons texte en graisse normale ; zones de dépôt en couleur d'accent de fond ; Répondre en icône seule en mode compact. P3 : « Ajouter aux favoris » ; historique du composeur sur toute la largeur. Décisions du propriétaire (2026-11-04) : compte en texte neutre multi-comptes ; bureau seulement (pas de mise en page téléphone) ; ligne À : maintenant ; plus d'icônes dans les menus. P2-3 tranché ensuite (option D) : Étiquettes au-dessus des comptes par défaut, compteurs de non-lus par étiquette |
| (flatpak.yml) | Workflow Flatpak à la demande ; premier bundle construit le 2026-11-04 (run 41892816974, 21,8 Mo). Snap de la même version : run 41862330599 |
| `ea980a2`, `1b5901a` | Snap : slot D-Bus pour le nom de l'application (AppArmor refusait le lancement) ; run 42397087858 vert, à vérifier par le propriétaire. Workflow AppImage à la demande ; premier bundle le 2026-11-07 (run 42397467139, 126 Mo) |
| `faf4529` | Développement des conversations dans la liste supprimé (réglages, lignes imbriquées, flèches) |
| `cb84d46` | Carte sélectionnée comme la maquette 01 : contour d'accent 2 px sur un fond teinté à 10 % ; liste sans le clavier : contour 1 px toujours en accent (plus de gris, #274). Message seul en carte par défaut (décision du 2026-11-07) ; les installations existantes gardent leur valeur (Réglages → Lecture) |
| `d73057c` | Pastilles d'initiales en aplat pastel, lettres foncées de la même teinte (maquette 01), mêmes couleurs et initiales dans la liste et le lecteur ; contour de sélection intérieur, de largeur égale (`01008df`) |
| `a48ae10` | Fond du volet de liste teinté comme la maquette 01 (mélange à 50 % fond de fenêtre / barre latérale, #f3f3f4 mesuré pour #f2f2f4) ; barres de recherche et de sélection gardent le fond de la fenêtre |
| `620ffdf` | Plus de trait entre deux bascules inactives du sélecteur Tous / Non lus / Favoris (absent de la maquette 01) |
| `a0c11c9` | Réglages → Étiquettes comme la maquette 09 (pastille, « n messages · Couleur », crayon, corbeille, « Nouvelle étiquette… », « Importer les étiquettes des serveurs… ») ; ajout et modification par la boîte « Nouvelle étiquette » (mode édition) ; mot-clé, couleur personnalisée et réordonnancement supprimés (décision du propriétaire, 2026-11-07). Sélecteur : seul le choix en gras, « Non lus ». `grab.sh` recompile le catalogue à chaque capture |
| `6d42417` | Nouvelle icône (docs/ICON.md) : rendu librsvg vérifié (masque, dégradés, flous corrects ; pin un peu plus net qu'avec Chromium), rien à simplifier ; `default.svg` remplacée, `default.Devel.svg` avec la bande de développement GNOME, `hicolor/` régénéré. Licence : AGPL-3.0-or-later (décision du 2026-11-07) |
| `131e215` | Galerie d'icônes supprimée (décision du 2026-11-07) : Réglages → Apparence → Icône, étape de l'assistant, redémarrage proposé, `data/icons/alt`, sources `classic`/`square*`, champs `app_icon*` de state.toml ; au démarrage, une icône choisie avant est retirée du lanceur |
| `06fab39` | Tri, affichage et filtres, étape 1 : « Non lus en premier », « Favoris en premier » (aussi le clic sur l'en-tête étoile) et le filtre « Étiquetés » supprimés (décisions du 2026-11-07). Le tri n'est pas enregistré : aucune migration ; une clé ancienne se lit « récents en premier » |
| `c756286` | Tri, affichage et filtres, étape 2 (maquettes 18 à 21) : popover « Trier et afficher » à une colonne (icône `tegami-sort-symbolic`), menu « Filtres » à côté du champ de recherche (Messages : 4 cases dont Importance haute ; Période : Toutes, Aujourd'hui, 7 j, 30 j, Cette année, Personnalisée…), pastilles retirables, « Tout effacer », badge et décompte, sélecteur Tous/Non lus/Favoris sous la recherche (`src/app/list_filters.rs`). Calendrier intégré remplacé par la boîte « Période personnalisée » ; `QuickFilter::Today` devient `Period::Today`. Crochet `TEGAMI_SHOWCASE_LIST=sort|filters|active|period`, captures 18–21 ajoutées à `mockup-shots.sh` |
| `b972dfe` | Popovers : ombre plus profonde et liseré plus net, comme les maquettes (coins de 12 px déjà identiques). Les captures passent par un compositeur (xcompmgr) et gardent l'alpha des popovers : coins et ombres visibles |
| `5867146` | Préférences, étape 1 (écrans 22 à 25) : `AdwPreferencesDialog` à 5 pages avec recherche (`src/ui/preferences/dialog.rs`, lignes gardées dans `pages/kept.rs`) ; pages Barre latérale, Conversations, Date et heure, Liste, Lecture, Système, Apparence retirées avec leur code, valeurs stockées gardées. Décisions du 2026-11-08 : extras de Rédaction sous « Avancé » (sous-page), son/boutons/connexion limitée retirés, restes des pages retirés, push global (Notifications), « Accusés de réception » (notification) gardé, étiquettes telles que `a0c11c9`, « Importance haute » gardée. À faire après l'étape 6 : relecture complète de DOCUMENTATION.md et FEATURES.md (plus de 90 renvois aux anciennes pages) |
| `fdecfcf` | Préférences, étape 2 (écran 24) : Confidentialité en 3 réglages. « Images et contenu distant » (Jamais/Demander/Toujours) lu depuis `auto_remote_content` + `show_remote_banner` (Toujours = les deux vrais) ; « Photos et logos » = `gravatar` et `sender_logos` (affiché actif si l'un l'est) ; accusés inchangés. `remote_ahead` et `show_spoof_banner` gardent leur valeur sans ligne. Sélecteur de pages : titres seuls (icônes masquées en CSS, libadwaita les exige) |
| `8477313` | Préférences, étape 3 (écrans 26 à 34) : liste des comptes en tête de la page Comptes (construite par le panneau des comptes, déplacée par le dialogue via son nom `accounts-list`), menu « Ajouter un compte… » (`context_menu`), page des comptes en ligne GNOME en sous-page ; fiche de compte courte (`pages/form.rs`) poussée sur le dialogue, six sous-pages (Serveurs, Synchronisation, Dossiers, Alias, Signatures, OpenPGP), champs rares sous « Avancé », test de connexion en icône + phrase, « Supprimer le compte » en ligne destructive, « Gérer les clés » ouvre les clés OpenPGP par-dessus. Cercle calculé : image, emoji, Gravatar retirés (valeurs gardées à l'enregistrement). Nettoyage des listes retiré du worker (`clean_lists_days` gardé). Libellés des sensibilités M365 retirés (valeur gardée). Compteurs étiquettes/filtres/dossiers intelligents ; sous-titres des services : étapes 4 et 5. Crochets démo `TEGAMI_SHOWCASE_ACCOUNT_SUB`, `TEGAMI_SHOWCASE_ADD_MENU` |

## Décisions prises

- Pousser directement sur `main` (choix du propriétaire).
- Valeurs par défaut changées pour suivre le design (les réglages restent,
  les fichiers déjà enregistrés gardent leurs valeurs) : teinte des lignes
  par étiquette désactivée ; étiquettes sous le texte ; messages lus
  repliés ; composeur et réponses en fenêtre ; historique affiché ;
  tiroir de pièces jointes désactivé ; barre du lecteur du design.
- Compteur de messages gardé dans l'en-tête de la liste, bien qu'absent
  des maquettes.
- Contrôles de pixels faits par captures du mode démo dans le conteneur ;
  contraste élevé et mode sombre non vérifiables là.
- Les nouveaux tests de la barre d'outils utilisent l'ancienne disposition
  par défaut comme jeu d'essai (`legacy_default`).

## Décisions du propriétaire (2026-11-02)

1. Menu principal : retirer Annuler, Volet de lecture, Mode Focus, Plein
   écran ; garder Imprimer, Quitter et les entrées de la maquette. Fait
   (`fd7a789`). Gardés aussi, non mentionnés : Afficher les comptes, barre
   d'état, Console.
2. Menu contextuel d'un message : celui de la maquette ; on verra en V2
   pour les fonctions retirées. Signaler toute fonction qui deviendrait
   inaccessible. Fait (`7d5bc0a`) ; seule « Copier vers… » n'a pas d'autre
   accès : question posée, gardée en attendant.
3. Compteurs par étiquette : laissés de côté.
4. Petites largeurs : hors de cette version, dans une version à part. Ne
   rien faire qui l'empêche plus tard (ne pas figer davantage la
   structure `OverlaySplitView` / `Paned` de `src/app.rs`).
5. Historique dans le composeur intégré : replié sauf le dernier message
   reçu. Fait (`583c960`).

## Décisions du propriétaire, suite (2026-11-02)

- « Copier vers… » : supprimé complètement (fait, `89e8aa0`).
- Supprimer les fonctions absentes des maquettes. Demandé explicitement :
  les deux interrupteurs de l'en-tête du lecteur, mode clair/sombre par
  message et vue Lecteur (faits, `15f9b65`, `bb5953c`). Relire les maquettes : le
  propriétaire voit dans les captures des éléments qui n'y sont pas.
- Ordre de travail convenu : 1. Copier vers (fait), 2. vérifier la CI,
  3. écarts avec les maquettes (barre latérale, champ De, menu ⋯ du
  composeur, menu ⋯ du lecteur, menu d'un dossier, avatars des
  destinataires, menu ⋯ d'une carte), 4. documentation (CHANGELOG,
  FEATURES, KEYBOARD_SHORTCUTS).

## Décisions du propriétaire, éléments absents des maquettes (2026-11-02)

- À supprimer complètement : vues unifiées de la barre latérale (Boîtes
  de réception, Favoris, Envoyés… de tous les comptes), section Filtres,
  lignes Contacts et Pièces jointes de la barre latérale (avec leurs
  pages) ; dans le composeur, le crayon du champ De (adresse libre, #347),
  la flèche d'Envoyer (Envoyer plus tard, #145), le bouton Fermer (X) de
  la fenêtre, le bouton Police « A » de la barre de mise en forme.
- À garder : puce du compte au-dessus de l'objet, puce « n destinataires »
  d'une carte, titres de groupes par date (non tranché, gardés).
- En-tête de la barre latérale fait (`3713aeb`) : titre « Courrier », pas
  d'Actualiser au début.

Note : le « bouton Police A » de la question était en fait le choix du
format (texte enrichi, brut, Markdown, HTML) ; erreur de description de ma
part, signalée au propriétaire. Le format est passé dans le menu ⋯ comme
dans la maquette 06.

Décisions du propriétaire (2026-11-02, suite) :
- Format : texte enrichi et texte brut seulement ; Markdown et Source HTML
  supprimés.
- Menu ⋯ du composeur : les 4 entrées absentes de la maquette sont
  supprimées (Enregistrer comme modèle, Envoyer vers le cloud, Ouvrir les
  contacts, Ouvrir dans une fenêtre).

Conséquences signalées : plus de création de nouveau modèle depuis le
composeur (les modèles existants s'ouvrent, se modifient et servent
toujours) ; l'envoi vers le cloud ne se fait plus que par la zone de dépôt
et « Envoyer un lien » d'une grosse pièce jointe ; l'emplacement du
composeur (fenêtre ou volet) ne se règle plus que dans les Réglages.

Comparaison maquettes / application : captures en français dans le
conteneur (locale `fr_FR.UTF-8` installée, catalogue compilé dans
`/tmp/loc`, `TEGAMI_LOCALEDIR=/tmp/loc LANGUAGE=fr LANG_=fr_FR.UTF-8`) ;
14 planches envoyées au propriétaire. Relevé : bouton Étiquettes absent de
l'en-tête du lecteur sur une conversation (planche 10), dates longues dans
la liste.

Nettoyage moteur de la galerie fait (schéma v22) : tables
`attachment_meta`/`attachment_scan` supprimées par migration, leurs
écritures et le préchargement des dossiers « galerie » retirés.

Conséquence à signaler au propriétaire : les contacts du carnet Tegami Mail
ne peuvent plus être modifiés ni supprimés un par un dans l'application
(seulement importés, exportés, tous supprimés) ; ceux des carnets GNOME ou
en ligne se modifient dans GNOME Contacts.

Vues unifiées retirées (`5022e1b`) : section du haut, réglages
`show_unified`/`unified_chips`/`unified_kinds`, replis du rail
correspondants, état d'ouverture dans `sidebar.toml`, Starred unifié des
comptes sans dossier Starred (#482), préchargement et préconstruction des
listes unifiées, actions « toutes les boîtes » du menu de dossier. Restent
les lignes Favoris, Étiquettes et Dossiers intelligents (moteur
`unified.rs` gardé pour elles) ; l'interrupteur de compte s'appelle
« Afficher dans les vues combinées » (`in_unified`). Ouverture au
démarrage : boîte du premier compte. Tray « Voir tous les non lus » :
première boîte avec du non-lu.

Conséquences à signaler : plus de Starred unifié (#482) ; « Afficher les
comptes » (Ctrl+Maj+A) et le mode Concentration « replier les lignes
unifiées » n'ont plus beaucoup de sens (gardés, à trancher).

Point 3 terminé (2026-11-02) :
- `e9fb982` menu ⋯ du lecteur selon la maquette 06, toujours affiché ;
  nouvelle action « Transférer en pièce jointe » (.eml joint, export brut
  `ExportRaw`, `pending_forward_attach`). Étoile, Reporter, Rechercher et
  Aperçu avant impression sortent de ce menu (restent au clic droit et
  aux touches).
- `a40a6c6` menu d'un dossier en sections (maquette 06) ; nouveau
  « Nouveau sous-dossier… » (`CtxAction::NewSubfolder`, `CreateFolder`
  avec `parent`). Les entrées hors maquette (Favoris, filtres, Masquer,
  Vider) sont gardées dans une section du milieu : à trancher avec le
  propriétaire.
- `552673d` avatar dans les puces de destinataires, bouton de retrait de
  32 px.
- `f3a334b` ⋯ d'une carte ouverte : Répondre, Répondre à tous, Transférer.

Remarques : une réponse met l'adresse nue dans « À » (pas le nom), donc
la puce montre l'adresse ; la maquette montre un nom. Les menus gardent
la casse des titres du catalogue (« Mark All as Read ») alors que
DESIGN.md demande la casse de phrase : à décider globalement.

## Décisions du propriétaire (2026-11-03)

1. Menu d'un dossier : les entrées hors maquette (Favoris, filtres,
   Masquer, Vider) sont conservées.
2. « Afficher les comptes » (menu, Ctrl+Maj+A, Réglages) et l'option
   Concentration « replier les lignes unifiées » : supprimés (`a0a871c`).
   L'option Concentration « masquer les comptes » reste.
3. Casse de phrase partout (`2546cac`) : script de conversion limité aux
   littéraux des appels `i18n…`, msgid renommés dans tous les catalogues
   (traductions gardées, doublons fusionnés), doc mise à jour. Règle
   ajoutée à CLAUDE.md. Le fichier .desktop garde sa casse de titre.
4. « Nom <adresse> » partout (`00b9d8f`) : réponse, réponse à tous
   (lecture par `parse_recipients`, écriture par `format_recipient`) et
   adresse cliquée dans le lecteur (nom du contact ou de la conversation).

## Fait le 2026-11-03 (suite)

- `2065f1f` mode Concentration retiré entièrement (décision du
  propriétaire) ; `5afe82f` les boutons des en-têtes, que son
  synchroniseur ouvrait au lancement, démarrent ouverts.
- `adb4bbf` dates courtes dans la liste (maquette 01) : heure, « Hier »,
  jour de la semaine, jour et mois, année si autre année.
- Catalogue français complet : les 93 entrées « fuzzy » relues (beaucoup
  étaient fausses) et les 61 chaînes manquantes traduites.

Décisions du propriétaire (2026-11-03, suite) : une seule section
Étiquettes (code par compte gardé derrière `ACCOUNT_TAG_SECTIONS`,
`0207791`) ; point de couleur dans les pastilles (`d72b305`, avec
l'opacité du texte secondaire du lecteur remontée à 70 %) ; contraste
élevé et mode sombre vérifiés plus tard par le propriétaire.

Décisions du propriétaire (2026-11-03, fin) : seule la fenêtre de
rédaction reste (`b3200db`, puis `454d8a5` pour les modes internes du
composeur) — réponse intégrée, réponse partagée, animation d'envoi
(`send_morph`), cartes provisoires et quatre réglages supprimés ; un
brouillon sélectionné s'affiche comme un message, Entrée ou double-clic
l'ouvre. « New message » dans le lanceur (`ace2d7c`). DESIGN.md parle
encore d'un réglage « en attendant une décision » : la décision est prise.

Comparaison maquettes 10 à 17 (2026-11-03) :
- 10 pièces jointes reçues : conforme (la taille « bytes » vient de GLib
  sans traductions dans le conteneur ; « octets » sur un GNOME français).
- 11 pièces jointes à l'envoi : cartes à bordure, nom en gras (`b73c2e5`).
- 12 dépôt de fichiers : cartes compactes empilées en haut (`a833a1b`).
- 13 pièce jointe oubliée : déjà conforme.
- 14 (1024×600) : la barre latérale passe en rail d'icônes, la maquette
  la replie entièrement derrière le bouton ; question au propriétaire.
- 15 (téléphone) : non pris en charge (largeur minimale 904 px) ; pile
  de navigation à décider avec le propriétaire.
- 17 grand texte : tient à 150 %.
- 16 contraste élevé : à vérifier par le propriétaire.

Décisions du propriétaire (2026-11-03, soir) : colonne d'icônes gardée
à 1024 px ; pas de version téléphone (application de bureau) ; message
seul à plat par défaut sur une nouvelle installation, revu le 2026-11-07 :
en carte par défaut (`single_message_card` à vrai ; une installation
existante garde sa valeur : Réglages → Lecture). Écart restant
avec la maquette 10 : l'en-tête d'un message seul montre la puce
« n destinataires » et la date à droite, la maquette « À : moi · date »
sous le nom.

## Reste à faire (connu)

- Les boutons des en-têtes et l'avatar des lignes restent dans des
  revealers (`focus-fade`) hérités du mode Concentration : la mesure du
  repli de la barre du lecteur s'appuie dessus (`follow_button`,
  `MeasureReaderFold`) ; les retirer demande de reprendre cette mesure.

- Vérifier contraste élevé et mode sombre sur une vraie session GNOME.
- Questions ouvertes de DESIGN.md (icônes symboliques, version sombre des
  écrans).
