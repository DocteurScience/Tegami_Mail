# Suivi du redesign

Journal de ce qui est fait, des décisions et de ce qui reste, pour
reprendre le travail dans une autre conversation. Les règles durables
(conventions, commandes, architecture) sont dans [CLAUDE.md](../CLAUDE.md).
Mettre ce fichier à jour à chaque étape poussée.

Dernière mise à jour : 2026-11-02, après `40a6c7e`.

## État

- Base : import de Hylki 1.46.0, renommé Tegami Mail (commits du
  2026-10-31 et du 2026-11-01 jusqu'à `fd0ccce`).
- Version : 1.0.0 (2026-11-11), première de Tegami Mail.
- CI : compilation et tests verts jusqu'au dernier commit ; le snap échoue
  depuis `c31a9e4` faute d'atteindre le magasin Snap (api.snapcraft.io,
  panne réseau côté GitHub, pas le code) : relancer quand il répond.
- Tests : 1078 réussis, 40 ignorés (la baisse suit les fonctions retirées
  par l'audit).
- Référence du design : `docs/DESIGN.md`, `docs/DESIGN_STRINGS.md`,
  maquettes `data/repo/design/01` à `48`.

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
| `83df670`, `5631585`, `ce58a5c`, `6a7d4cb`, `39b63ef` | Audit du code (2026-11-03) : clippy `--all-targets` à zéro ; entrées obsolètes des catalogues purgées (`update-pot.sh` les retire désormais) ; 27 règles CSS et 5 icônes mortes retirées ; actions rapides du tiroir visibles au focus clavier ; Ctrl+Maj+A mort et relais `ZoomKey`/`ComposerKey` retirés ; ~40 commentaires périmés (All Inboxes, réponse dans le volet, galerie) ; panique IMAP corrigée (lecture sans session après un envoi de brouillon). Restés à décider, réglés le 2026-11-10 : vues unifiées simplifiées, touches Gmail « dans une fenêtre » ramenées à leurs équivalents (voir plus bas) |
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
| `0de9515` | Préférences, étape 4 (écrans 35 à 45) : listes avec ligne « Ajouter… » en dernier (helpers `ui::accounts::{add_row, action_row}`, classe CSS `add-row`), suppression dans l'éditeur (filtre, dossier intelligent, compte de stockage), « Appliquer maintenant » en ligne, recherche des filtres à partir de 8. « Avancé » : étiquette + compte des non lus (filtre), code à deux facteurs + Cloudflare (stockage), serveur + région (traduction). OpenPGP : listes avec importer/générer/chercher, note GnuPG en bas. « Proposer de traduire » retiré (`offer` gardé, plus lu). Sous-titres Stockage et Traduction sur la page Comptes. Étiquettes laissées telles que `a0c11c9` (décision du propriétaire). La barre latérale avait déjà la section repliable « Dossiers intelligents ». Critères rares sous « Avancé » : fait ensuite (refonte des éditeurs) |
| `170e4d4` | Préférences, étape 5 (écrans 46 à 48) : carnet Tegami Mail retiré (lecture, écriture, groupes, photos, import vCard, export, « tout supprimer ») ; `contacts/local.rs` réduit à l'ancien carnet en lecture seule (compter, exporter) ; `app/legacy_book.rs` propose l'export vCard au démarrage tant que le carnet contient des contacts, puis renomme le fichier en `local_contacts-exported.db` (jamais supprimé). « Afficher les noms des contacts » toujours actif (valeur stockée gardée). Sans Evolution Data Server, plus de carnet où enregistrer un contact. Carnets : ligne « Ajouter un carnet d'adresses… » en fin de liste, lignes Ouvrir Contacts / Comptes en ligne. Sauvegarde : confirmation avant import. LDAP « Avancé » fait ensuite |
| `af609bd` | Préférences, étape 6 : entrée « Dépannage… » du menu principal (`app/troubleshooting.rs`, `adw::Dialog`) : barre d'état affichée/en haut, mode console, ouvrir la console, exporter le journal. « Préférences » remplace « Paramètres » dans le menu. Crochet `TEGAMI_SHOWCASE_TROUBLESHOOTING`. Reste : relecture complète de DOCUMENTATION.md et FEATURES.md |
| `7c23e1c` | Relecture de DOCUMENTATION.md et FEATURES.md pour les nouvelles préférences : chemins « Préférences → … » pour les réglages gardés ; passages sur les réglages retirés supprimés ou réécrits (comportement par défaut, « une valeur d'une version précédente est gardée ») : barre latérale, barre d'outils, disposition et colonnes de la liste, taille du texte, icônes, compteur sur l'icône, moteur de rendu, Fichiers GNOME (installation manuelle seulement), son et boutons des notifications, connexions limitées, carnet Tegami Mail, nettoyage des listes, libellés de sensibilité |
| `3192415` | Formulaire LDAP (écran 47) : chiffrement, port, base et portée de recherche sous « Avancé » (ouvert quand l'annuaire a déjà une base) |
| `8884588` | Éditeurs de filtres et de dossiers intelligents refaits (écrans 38 et 40) : `accounts/criteria.rs` (liste de critères partagée : « Critères : tous / un seul », une ligne par critère titrée « champ + opérateur », bouton ⋯ pour l'opérateur ou retirer, « Ajouter un critère », critères rares ajoutés depuis « Avancé ») remplace `condition_editor.rs` ; `action_editor.rs` : une ligne par action, menu « Ajouter une action » limité à ce qui manque ; dossier intelligent : lignes fixes Lecture / Favoris / Pièce jointe / Étiquette, critères de texte gardés. Format enregistré inchangé. Limite connue : un dossier intelligent qui aurait deux états du même groupe (Lus et Non lus) n'en montre qu'un |
| `a09a089` | Evolution Data Server noté comme dépendance d'exécution pour les contacts (BUILDING.md, CHANGELOG). Note périmée retirée de « Reste à faire » : les revealers du mode Concentration ont été retirés par `80e9d01` |
| `891b889` | Assistant de premier lancement (écran 49) : `AdwNavigationView` à la place du carrousel et de ses points, retour et fermeture dans l'en-tête, couleurs du système (clair, sombre, contraste élevé vérifiés) ; pages langue, confidentialité et « Personnaliser » retirées, CSS `.welcome-*` et maintien du thème clair supprimés, ainsi que l'aide au redémarrage (qui ne servait plus qu'à la langue) ; fermer l'assistant vaut fin ; `wordmark.rs` passe dans `src/ui/` (À propos, avis de reprise) ; crochet `TEGAMI_SHOWCASE_WIZARD`(`_SHOT`) |
| `c57cb7a` | Écran 50 « Ajoutez vos comptes » : liste des comptes en ligne et des comptes ajoutés à la main, « Ajouter » devient coche + « Ajouté » ; groupe « Un autre compte » (Compte e-mail…, Ouvrir Comptes en ligne) ; nouvelle recherche des comptes au retour sur la fenêtre (bouton « Rechercher à nouveau » retiré) ; `TEGAMI_SHOWCASE_WIZARD=accounts` montre les comptes de la maquette |
| `ee74104` | Écran 51 « Compte e-mail » : adresse et mot de passe, « Avancé » replié (fournisseur, nom affiché, serveurs, client OAuth ou jeton selon le fournisseur) ; fournisseur déduit de l'adresse (`provider_for_address`, testé) tant qu'il n'est pas choisi à la main ; test : indicateur + « Vérification de la connexion… », bouton désactivé ; erreur en phrase sous les champs ; succès : retour à l'écran 50. Rien de retiré : Microsoft (#329), OAuth personnalisé et JMAP restent dans « Avancé » |
| `8ef249d` | Écran 52 « Tout est prêt » : icône à 96 px, phrase qui nomme le premier compte ajouté, « Lire mon courrier », pas de retour ; sans compte ajouté, « Continuer » ferme l'assistant ; lien vers les tickets seulement dans À propos ; `TEGAMI_SHOWCASE_WIZARD=done`. Les quatre écrans comparés aux maquettes en clair, sombre et contraste élevé |
| `a85bd8b` | Corrections trouvées en faisant tourner les scénarios de démo : 40 avertissements GTK par ouverture des préférences (icônes du sélecteur à taille 0, maintenant 1 px transparentes), règle CSS `:not(:empty)` jamais lue (erreur d'analyse à chaque démarrage), `PROVIDERS` et `PREDECESSORS` passés de `const` à `static` et comparés par nom (la détection du fournisseur dans l'assistant pouvait échouer) |
| `e25ab36` | Ménage : moteur de thèmes colorés retiré (`desktop/theme.rs`, `theme_palettes.rs`, `tools/gen-themes.py`, champ `theme`), palette d'actions au survol des lignes de la liste retirée (`message_row/palette.rs` réduit au menu et au glisser, réglages `list_palette*` et `palette_collapse_secs`, CSS, crochet `TEGAMI_SHOWCASE_PALETTE`), et ce qui ne servait qu'à eux (`color::darken`, `SetTagFor`, `RowMoveTo`) ; 5 icônes inutilisées hors du paquet, source de `tegami-sort` ajoutée pour que `gen-icon-gresource.sh` aboutisse |
| `f5ae7b5` | Palette d'actions des cartes du lecteur retirée (décision du 2026-11-10, captures envoyées) : le ⋯ ouvre toujours le menu ; rangée `tegami-acts` (HTML, script, CSS), réglages `card_actions_hover`, `card_actions_auto`, `card_palette_menu`, `card_palette_collapse_secs`, verbes du pont réservés à la palette et ce qui n'était construit que par eux (`ToggleRemote`, `CardContact`, `CardMoveAt`, `ContactSender`, `CardMoveTo`…) ; instantanés HTML régénérés |
| `19ffc89` | Moteur des polices et couleurs propres du lecteur retiré (#56, #181 ; « Reading: own font and colours ») : `ReaderStyle`, `reader_style_css`, `css_font`, `SetReaderStyle`, le choix « formatage de l'expéditeur » par carte, les réglages `override_fonts`, `reader_font`, `override_colors`, `plain_monospace`, `plain_font` ; l'injection CSS reste pour le zoom. Instantanés HTML inchangés, lecteur identique au pixel avec ou sans anciennes valeurs |
| `9caa6c5` | Audit « Removed or moved », lot 1 (liste) : réglages sans ligne mais encore lus retirés et figés à leur valeur par défaut — `list_layout` (cartes à côté du lecteur, tableau en dessous ou sans lecteur, #475 gardé), `list_group_by_date`, `list_pills_below`, `list_search_bottom`, `list_bulk_bar_bottom`, `swipe_reversed`, `swipe_sensitivity` (constante 3,5), `toolbar_fold_right` ; migrations `date_groups_default_applied`. Fenêtre par défaut identique au pixel |
| `999db81` | Audit, lot 2 (lecteur) : `reader_find_bottom`, `always_show_recipients`, `account_chip`, `pgp_labels`, `single_message_card` (+ migration), `fold_messages`, `thread_row_newest` (#236, avec `latest_elsewhere`), `message_theme`, `show_spoof_banner`, `reader_zoom` retirés et figés ; `apply_new_defaults` vide supprimée. Trois instantanés HTML changés, et seulement là où ils testaient des modes non par défaut (message seul en pleine largeur, destinataires toujours ouverts) ; fenêtre par défaut identique au pixel |
| `552803a` | Audit, lot 3 (barre latérale) : `sidebar_hover_expand`, `remember_sidebar`, `remember_rail`, `rail_dots`, `seen_counts` (#343, avec `unread_seen`, `fresh_unread`, `restyle_seen`, son CSS), `rail_fold`, `section_style` (style arbre), `tags_placement`, `favorites_placement` (avec la migration `LegacySwitches`), `account_click_folds` retirés et figés. Reste `chevrons_left` (17 sites), à faire à part. Fenêtre par défaut identique au pixel. Constat : la suppression d'une pièce jointe du serveur n'est atteignable que par le tiroir de pièces jointes, désactivé par défaut et sans réglage — question au propriétaire |
| `17555af` | Audit, lot 3 bis : `chevrons_left` retiré (chevrons à droite, comme par défaut) — 17 sites du dessin de la barre latérale, le paramètre `inset` des constructeurs de lignes, `ROW_LEFT_INSET` et le CSS de la variante gauche. Fenêtre et rail identiques au pixel |
| `bf7fddd` | Audit, lot 4 (application) : `start_fullscreen`, `metered_save_data`, `desktop_icons`, `remote_ahead` (#474, avec la file « ahead » du téléchargement, `fetch_ahead`, `markup_url`, `remote_urls`, le champ `ahead` de la sélection), `settings_open_accounts`, `tag_row_tint`, `link_browser` (#232, choix du navigateur), `list_search_open` (#446), `own_mailbox_face`, `contact_names` retirés. Fenêtre et recherche identiques au pixel |
| `c572cfc` | Décision du propriétaire (2026-11-10) : « Supprimer du serveur… » ajouté au menu ⋯ des pièces jointes des cartes (en rouge, même confirmation, testé par clics) ; tiroir de pièces jointes retiré (`ui/attachment_drawer/`, `grab_pill.rs`, l'état du tiroir dans state.toml, `card_attachments`, `ScrollToAttachments`, les vignettes asynchrones) ; `previewable` passe dans `thumbnails.rs`. Fenêtre par défaut et conversation à pièces jointes identiques au pixel. Jeu d'icônes : le propriétaire hésite, recommandation de garder le jeu propre et de corriger DESIGN.md |
| `be9425c` | DESIGN.md : les icônes sont le jeu Adwaita propre à l'appli, embarqué, quel que soit le thème du bureau (décision du propriétaire, 2026-11-10) |
| `6a08c40` | Audit, lot 5 : icône de la zone de notification retirée (`app/tray/indicator.rs`, réglages `tray`, `tray_icon`, `tray_mail`, messages `QuitFromTray`, `TrayViewUnread`, dépendance `ksni`, permission Flatpak `StatusNotifierWatcher`, `tray_image`, `custom_icon`, `desktop_icon_theme`) ; `tray.rs` ne garde que les compteurs (barre latérale, Applications en arrière-plan, pastille du lanceur). Session : disque plein, cache incrémental vidé |
| `c9bf711` | Audit, lot 6 : styles de date et d'heure retirés (`date_style`, `clock_style`, `DateStyle`, `ClockStyle`, `set_style` et ses atomiques dans `datefmt.rs`). Les dates suivent toujours la langue du système et le format d'heure du bureau GNOME (décision du propriétaire, 2026-11-10). Vérifié : avec les anciennes valeurs `yearfirst` / `twelve` enregistrées, la fenêtre est identique au pixel à celle par défaut |
| `fd1b76d` | Audit, lot 7 : taille du texte propre à l'appli retirée (`text_scale`, `desktop/text_scale.rs`, `load_text_scale`). Le texte suit toujours la taille du bureau (décision du propriétaire, 2026-11-10). Vérifié : avec `text_scale = 150` enregistré, la fenêtre est identique au pixel à celle par défaut |
| `23ed4af` | Audit, lot 8 : vue de démarrage retirée (#256 : `start_view`, `StartView`, `start_target`, `StartTarget` et l'attente de son compte dans la barre latérale, `last_view` / `last_account` de state.toml écrits à chaque changement de dossier) ; `DropStart` devient `DropReannounce`, seul usage restant. L'appli s'ouvre toujours sur la boîte de réception du premier compte (décision du propriétaire, option A, 2026-11-10, ajoutée à DESIGN.md). Vérifié : avec `start_view = "last_folder"` enregistré, fenêtre identique au pixel |
| `1af948a` | Audit, lot 9 : choix des boutons de notification retiré (#244 : `notification_buttons`, `NotificationButton(s)`, `load_notification_buttons`) ; les boutons Répondre, Transférer et Indésirable, que seul ce choix ajoutait, partent avec leurs actions (`REPLY/FORWARD/SPAM_ACTION`, `AppMsg::Notification{Reply,Forward,Spam}`, `pending_notified_reply`). Une notification d'un seul message porte toujours Marquer comme lu, Archiver, Supprimer (décision du propriétaire, option A, 2026-11-10, ajoutée à DESIGN.md). Catalogues remis à jour (chaînes de l'icône de notification retirée au lot 5). Docs dans le commit suivant |
| `18af73e` | Audit, lot 10 : choix de la pastille du lanceur retiré (#271, #431 : `launcher_count`, `launcher_badge`, `LauncherBadge`, le suivi `Arrivals` et `note_inbox_arrivals`). La pastille compte toujours les non-lus des boîtes de réception, là où le dock l'affiche (décision du propriétaire, option A, 2026-11-10, ajoutée à DESIGN.md) ; `WindowInFront` ne sert plus qu'à la renvoyer après un déverrouillage |
| `1032287` | Audit, lot 11 : colonnes du mode tableau retirées (#334, #443 : `list_columns`, `list_headings`, `list_column_scroll`, `list_column_widths`, les en-têtes cliquables et leurs poignées, `fill_headings`, `column_sort`, `sort_reversed`, `SortByColumn`, `ResizeColumn`, `ListColumnWidths`, `ListSortChanged`, le défilement horizontal, `min_width`, le CSS `.list-headings`). `message_row/headings.rs` devient `labels.rs`. Le tableau garde les colonnes par défaut, ajustées au panneau (suppression approuvée dans DESIGN.md, « Message list: columns »). Vérifié au pixel : tableau par défaut et cartes inchangés, anciennes valeurs sans effet. Reste mort : les colonnes et ordres de tri que seuls les en-têtes atteignaient |
| `463d576` | Audit, lot 12 : code devenu inaccessible après le lot 11. Colonnes destinataires, correspondants, importance, compte et échéance (variantes de `ListColumn`, widgets de ligne, `fill_columns`, `correspondents`, `people_of`, `due_label`, `SetAccountNames` et l'état Microsoft 365 qui ne servait qu'à l'échéance, CSS, icône `tegami-importance-low-symbolic`) ; huit ordres de tri que seuls les en-têtes atteignaient (`SortOrder` garde date ↓/↑, expéditeur, sujet ; `tag_sort_key`, `text_key_cmp`, tri par correspondants). Aucun changement visible : tableau et cartes identiques au pixel |
| `5d2291d` | Vues unifiées simplifiées (décision du propriétaire, 2026-11-10) : le moteur ne sert plus qu'aux Favoris. `UnifiedView` (un seul cas) et `unified_account` (toujours `None`) retirés ; `app/unified.rs` devient `app/favorites_view.rs`, champs, fonctions et messages renommés (`favorites_open`, `favorites_slices`, `open_favorites`, `AppMsg::FavoritesSelected`…), commentaires périmés (All Inboxes, Starred, Snoozed unifié, Archive) corrigés. Le nom côté barre latérale (`UnifiedRow`, section des Étiquettes et Dossiers intelligents) et l'interrupteur de compte `in_unified` sont gardés. Vérifié au pixel : Favoris ouvert et fenêtre par défaut identiques (le mode démo n'a pas de dossier favori, la fusion n'est couverte que par les tests) |
| `9d47316` | Touches Gmail « dans une fenêtre » (décision du propriétaire, 2026-11-10) : tout composeur est une fenêtre, donc `d`, `Shift+R`, `Shift+A`, `Shift+F` deviennent de simples équivalents de `c`, `r`, `a`, `f`. `Shortcut::InWindow`, `WindowCompose` et `AppMsg::ComposeInWindow` retirés ; liste des raccourcis (« r / Shift+R », « c / d ») et KEYBOARD_SHORTCUTS.md mis à jour, trois chaînes retirées. Vérifié par capture de la liste en mode Gmail. Constat : dans cette liste, « or » entre deux touches (« o or Enter ») n'est pas traduit |
| `db53c85` | Liste des raccourcis : le « or » entre deux touches (« j  or  ↓ ») passe par `i18n_f("{first}  or  {second}")` (`key_label` dans `app/palette.rs`, commentaire pour les traducteurs, test) et est traduit dans les 11 langues (ca o, de oder, el ή, es o, fr ou, hu vagy, it o, pl lub, pt ou, ru или). Vérifié par capture en français ; `grab.sh` ne compile que le catalogue français, les autres sont validés par `msgfmt --check` |
| `cf905a1` | Icône (ICON.md, point 5) vérifiée : plus aucune trace de l'icône Hylki (fichiers d'icône, logos, README, AppStream). La fenêtre À propos n'affichait aucune icône (le logo « avec icône » n'était que du texte) : l'icône de l'appli, 128 px, passe au-dessus du nom ; `Wordmark::WithIcon` devient `Wordmark::Name`. « What's new in {version} » et « Hylki (original project) » passent par i18n, traduits dans les 11 langues. Vérifié par captures en clair et en sombre. Restent : description AppStream périmée, pas de captures d'écran dans la fiche, développeur Hyprlab dans la fiche (au propriétaire) |
| `4b1fa30` | Fiche AppStream réécrite (validée par le propriétaire, 2026-11-11) : la description ne cite plus les fonctions retirées (Toutes les boîtes, mode Lecture, thèmes, mode Concentration, icône de notification, choix de langue et d'horloge) ; résumé raccourci en « A fast, private email client for GNOME » (`appstreamcli validate` sans avertissement) ; « GNOME on Wayland » devient « GNOME ». Les 15 textes traduits dans les 11 langues, aucune entrée floue ni vide. Restent pour la fiche : captures d'écran, développeur et contact (Hyprlab, au propriétaire) |
| `8d96204` | État des catalogues (2026-11-11), 1751 chaînes : fr complet ; de 1409 actives (237 floues, 105 vides), pt_BR/pt_PT 1350, ca 1336, el/es 1335, it 1141, ru 1025, hu 896, pl 853. Les floues ne s'affichent pas (anglais à la place) et sont souvent de fausses pistes de msgmerge (ca « Read receipts » → « Amaga els destinataris »). Aucune variable fausse dans les traductions actives, sauf hu « {n} file · {size} » qui affichait « {méret} » : corrigé (« fájl » au singulier après un nombre, en hongrois) |
| `ef43824` | Traductions complétées (option B du propriétaire, 2026-11-11) : les 898 textes manquants ou flous traduits dans les 10 langues autres que le français (5 480 traductions ; ru et pl avec leurs trois formes de pluriel), en reprenant les termes déjà employés par chaque catalogue ; les fausses suggestions de msgmerge sont remplacées. Les 11 catalogues sont à 1 751/1 751. Contrôle automatique : aucune variable `{…}` fausse ni « … » perdu, `msgfmt --check` propre. Vérifié par captures en allemand et en russe. Traductions faites sans relecture par des locuteurs natifs : à faire relire par les traducteurs (en-têtes des catalogues). Constat : en allemand, « Archive » est un seul texte pour le dossier et l'action, traduit « Archivieren » (verbe) |
| `3d39b35` | Noms des dossiers spéciaux avec un contexte gettext (`i18n_c`, `msgctxt "folder name"`, `-k i18n_c:1c,2` dans `update-pot.sh`) : « Archive » était un seul texte pour le dossier et l'action, et le dossier s'affichait « Archiver » en français, « Archivieren » en allemand ; il s'affiche « Archives », « Archiv »… dans les 11 langues. Faute hongroise corrigée (« Csillakerülzott » → « Csillagozott »). Vérifié par captures fr et de |
| `328013a` | Préférences vérifiées en allemand et en russe (captures) : les onglets, de largeur égale, coupaient « Benachrichtigungen », « Учётные записи », « Уведомления », « Конфиденциальность ». Termes plus courts et courants : de « Mitteilungen », ru « Аккаунты », « Оповещения », « Приватность ». Les autres langues restent sous la longueur du français, qui tient. Composeur et assistant non capturés dans ces langues (les crochets de capture ne les montrent pas sous `grab.sh`) |
| `e66bd6b` | Composeur et assistant (accueil, formulaire, fin) capturés en allemand et en russe (`TEGAMI_WELCOME=1 TEGAMI_SHOWCASE_WIZARD=… TEGAMI_SHOWCASE_WIZARD_SHOT=…`, et `DELAY` court pour le composeur) : rien ne déborde. Titre du composeur allemand « Marcus Chen antworten » → « Antwort an Marcus Chen ». Constat : en russe, l'onglet dit « Аккаунты » (pour tenir) quand le reste dit « учётные записи » |
| `4347a6f` | Libellés accessibles : balayage des boutons à icône (construits en code et dans les vues relm4). Seul manque réel : la croix des notifications dans l'appli, qui reçoit « Dismiss » (traduit dans les 11 langues). Les deux boutons OpenPGP du composeur sans libellé ne sont jamais affichés (ils gardent l'état). Questions closes dans DESIGN.md : étiquettes accentuées (`unique_keyword`, testé) et icônes |
| `86522a7` | Fiche AppStream (propriétaire, 2026-11-11) : développeur DocteurScience (`io.github.docteurscience`), contact tirr.mohma@gmail.com ; quatre captures en anglais dans `data/screenshots/` (liste, sombre, composeur, préférences, en 2x), citées par URL raw GitHub, légendes traduites dans les 11 langues ; `appstreamcli validate` propre, fiche traduite générée par `msgfmt --xml`. Les comptes de démo des captures portent encore le nom Hyprlab (données du mode démo) |
| `7c8eb89` | Compte de démo « Hyprlab » (hello@hyprlab.dev) renommé « Pinecone Lab » (hello@pinecone.dev), avec ses messages ; PRODID des réponses d'invitation → `-//DocteurScience//Tegami Mail//EN`. Les quatre captures de la fiche refaites |
| `90529c7` | Batterie de 69 scénarios en mode démo (préférences et sous-pages, menus, composeur, assistant, Favoris, lecteur dessous/masqué, touches Gmail, sombre, contraste élevé, de/ru, anciennes valeurs des réglages retirés) : aucune erreur critique, panique, erreur CSS ni avertissement. Un défaut vu sur les captures : un message ouvert dans sa fenêtre s'ouvrait le sujet sélectionné (le libellé sélectionnable prenait le focus initial) ; le sujet sort de la chaîne de focus, sélection à la souris et copie par clic droit gardées. Fenêtre principale identique au pixel |
| `5fd9229` | Version 1.0.0 (propriétaire, 2026-11-11 : on repart de 1.0.0) : `Cargo.toml`, `Cargo.lock`, snap ; fiche AppStream réduite à une version 1.0.0 (l'historique de Hylki reste dans CHANGELOG et RELEASE_NOTES) ; notes « What's new in 1.0.0 » rédigées d'après le CHANGELOG (affichées dans À propos) ; « Unreleased » du CHANGELOG devient 1.0.0. Aucun code ne compare de versions (seule une clé de cache des polices de l'AppImage, par égalité) |
| `7985aca` | Trait entre deux bascules inactives du sélecteur Tous / Non lus / Favoris rétabli (décision du propriétaire, 2026-11-11 : le masquer contrevenait au HIG ; annule `620ffdf`). Le séparateur de libadwaita s'affiche de nouveau |
| `89e166b` | Icône de l'assistant et d'À propos : la copie embarquée (`resources/icons/scalable/apps/`) était restée l'icône temporaire de `a6bba70` quand `6d42417` a changé l'icône (signalé par le propriétaire sur sa machine). Copie mise à jour, test qui compare les deux ; l'assistant et À propos chargent l'icône par un thème privé limité aux ressources (`wordmark::app_icon_image`), qu'une ancienne icône installée ne peut plus masquer. Vérifié par capture avec l'ancienne icône dans un dossier d'icônes système |
| `e1919da`, `6731230`, `30483b6` | Retours du propriétaire sur la 1.0.0 : le lien « Ouvrir les réglages de notification du système » passe par D-Bus (`desktop::settings_panel`, partagé avec Comptes en ligne) au lieu de lancer `gnome-control-center`, impossible en Flatpak et cassé en AppImage ; onglets des Réglages à la largeur de leur nom (« Confidentialité » n'est plus tronqué, vérifié en français et en allemand) ; menus : le focus suit le pointeur, une seule ligne marquée, sans anneau sur les cases (`focus_follows_pointer`) ; plus de disque gris au survol des chevrons du volet latéral. Le composeur (arrondis) et les cases rondes vertes des captures du propriétaire ne se reproduisent ni sous Adwaita ni sous Yaru : thème installé chez lui, question posée |
| (ce commit) | Version 1.0.1 : icône, lien des notifications, onglets des Réglages, menus, chevrons. Section « What's new in 1.0.1 », version AppStream, CHANGELOG ; publication lancée par `release.yml` |

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

- Vérifier contraste élevé et mode sombre sur une vraie session GNOME.
- Plus de question ouverte dans DESIGN.md : les maquettes sombres ne sont
  pas nécessaires (propriétaire, 2026-11-11). Réglées : les icônes (jeu propre à l'appli, 2026-11-10) et
  les étiquettes aux noms accentués (`Tag::unique_keyword` donne « Ete_2 »
  à « Ete » quand « Été » existe, testé dans `config/filter_tests.rs`).
- Étiquettes (écrans 35, 36) : laissées telles que `a0c11c9` par décision
  du propriétaire (pas de réordonnancement, de mot-clé ni de couleur
  personnalisée) ; à rouvrir seulement s'il le demande.
