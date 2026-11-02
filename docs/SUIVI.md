# Suivi du redesign

Journal de ce qui est fait, des décisions et de ce qui reste, pour
reprendre le travail dans une autre conversation. Les règles durables
(conventions, commandes, architecture) sont dans [CLAUDE.md](../CLAUDE.md).
Mettre ce fichier à jour à chaque étape poussée.

Dernière mise à jour : 2026-11-02, après `89e8aa0`.

## État

- Base : import de Hylki 1.46.0, renommé Tegami Mail (commits du
  2026-10-31 et du 2026-11-01 jusqu'à `fd0ccce`).
- CI : verte sur tous les commits jusqu'à `3e900c1` (build, tests, snap) ;
  les suivants à vérifier (`gh run list --repo DocteurScience/Tegami_Mail`).
- Tests : 1243 réussis, 45 ignorés.
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
  message et vue Lecteur (en cours). Relire les maquettes : le
  propriétaire voit dans les captures des éléments qui n'y sont pas.
- Ordre de travail convenu : 1. Copier vers (fait), 2. vérifier la CI,
  3. écarts avec les maquettes (barre latérale, champ De, menu ⋯ du
  composeur, menu ⋯ du lecteur, menu d'un dossier, avatars des
  destinataires, menu ⋯ d'une carte), 4. documentation (CHANGELOG,
  FEATURES, KEYBOARD_SHORTCUTS).

## Reste à faire (connu)

- « Copier vers… », selon la réponse.
- Section Étiquettes de la barre latérale ouverte par défaut (les valeurs
  par défaut de `sidebar.toml` valent pour toutes les sections ; un test
  fige ce comportement).
- Vérifier contraste élevé et mode sombre sur une vraie session GNOME.
- Environ 100 traductions françaises « fuzzy » héritées de Hylki.
- Questions ouvertes de DESIGN.md (icônes symboliques, version sombre des
  écrans).
