# Design strings

The text the [design](DESIGN.md) needs, in English as it goes in the code, with
a proposed French translation. The third column says whether the string is
already in `po/tegami.pot`, and if so what `po/fr.po` has for it.

How to use it:

1. Look for a near match in `po/tegami.pot` before adding a string. A string
   marked "new" here may exist under a slightly different wording, and
   reusing it keeps the catalogue small and the translations consistent.
2. Put the English in the code, through `i18n`, `i18n_f` (named placeholders,
   written `{name}`) or `ni18n` (plurals). Never build a sentence by joining
   pieces: a translator needs the whole sentence to reorder it.
3. Run `tools/update-pot.sh`, then add the French to `po/fr.po`. The other
   languages show the English until their translators update them.
4. Counts are plurals. "{n} attachments", "{n} messages" and "{count} files"
   go through `ni18n`, so French "0 or 1 attachment" and Russian's three forms
   come out right.

The French here is a proposal for review, not a final text. Where the
catalogue already translates a string differently (for example "Tags" is
"Tags" today, and the design uses "Étiquettes"), the design's choice is
noted in [DESIGN.md](DESIGN.md#code-and-translations).

Keyboard shortcuts are not translated.

### Main window

| Source string (English) | Proposed French | In the catalogue |
| --- | --- | --- |
| New Message | Nouveau message | yes, French: Nouveau message |
| Reply | Répondre | yes, French: Répondre |
| Reply to all | Répondre à tous | yes, French: Répondre à tous |
| Forward | Transférer | yes, French: Transférer |
| Archive | Archiver | yes, French: Archiver |
| Delete | Supprimer | yes, French: Supprimer |
| Tags | Étiquettes | yes, French: Tags |
| More actions | Plus d’actions | new |
| Main menu | Menu principal | new |
| Search | Rechercher | yes, French: Rechercher |
| Sort and view | Trier et afficher | new |
| Show folders | Afficher les dossiers | new |
| Back | Retour | new |
| All | Tous | new |
| Unread | Non lus | yes, French: Non lu |
| Starred | Favoris | yes, French: Favoris |

### Main menu

| Source string (English) | Proposed French | In the catalogue |
| --- | --- | --- |
| Sync now | Synchroniser maintenant | new |
| Add Account… | Ajouter un compte… | yes, French: Ajouter un compte… |
| GNOME Online Accounts… | Comptes en ligne GNOME… | new |
| Preferences | Préférences | new |
| Keyboard Shortcuts | Raccourcis clavier | yes, French: Raccourcis clavier |
| Help | Aide | new |
| About Tegami Mail | À propos de Tegami Mail | yes, French: À propos de Tegami Mail |

### Message menus

| Source string (English) | Proposed French | In the catalogue |
| --- | --- | --- |
| Forward as attachment | Transférer en pièce jointe | new |
| Mark as unread | Marquer comme non lu | yes, French: Marquer comme non lu |
| Mark as read | Marquer comme lu | yes, French: Marquer comme lu |
| Mark as spam | Marquer comme indésirable | yes, French: Marquer comme indésirable |
| Move to… | Déplacer vers… | yes, French: Déplacer vers… |
| Show remote images | Afficher les images distantes | new |
| View source | Afficher la source du message | yes, French: Afficher le code source |
| Print… | Imprimer… | yes, French: Imprimer… |
| Save as… | Enregistrer sous… | new |
| Star | Ajouter aux favoris | yes |

### Sort, display and filters

Screens 18 to 21. Most of these strings are in the catalogue already: reuse
the catalogue's wording where the mockup differs ("Has attachments" for
"Avec pièce jointe", "Addressed to me" for "Adressés à moi", "From" and "To"
for "Du" and "Au").

| Source string (English) | Proposed French | In the catalogue |
| --- | --- | --- |
| Sort by | Trier par | yes, French: Trier par |
| Date (Newest first) | Date (récents en premier) | yes, French: Date (récents en premier) |
| Date (Oldest first) | Date (anciens en premier) | yes, French: Date (anciens en premier) |
| Sender (A–Z) | Expéditeur (A–Z) | yes, French: Expéditeur (A–Z) |
| Subject (A–Z) | Objet (A–Z) | yes, French: Objet (A–Z) |
| Show | Afficher | yes, French: Afficher |
| Group into conversations | Regrouper en conversations | yes, French: Regrouper en conversations |
| Message preview | Aperçu du message | yes, French: Aperçu du message |
| Avatars | Avatars | yes, French: Avatars |
| Filters | Filtres | yes, French: Filtres |
| Messages | Messages | yes, French: Messages |
| Has attachments | Contient des pièces jointes | yes, French: Contient des pièces jointes |
| Addressed to me | Adressé à moi | yes, French: Adressé à moi |
| From contacts | De mes contacts | yes, French: De mes contacts |
| Period | Période | new |
| Any date | N’importe quelle date | yes, French: N’importe quelle date |
| Today | Aujourd’hui | yes, French: Aujourd’hui |
| Last 7 days | 7 derniers jours | yes, French: 7 derniers jours |
| Last 30 days | 30 derniers jours | new |
| This year | Cette année | new |
| Custom… | Personnalisé… | yes, French: Personnalisé… |
| Clear filters | Effacer les filtres | yes, French: Effacer les filtres |
| Clear all | Tout effacer | yes, French: Tout effacer |
| Remove filter: {name} | Retirer le filtre : {name} | new |
| Filters, {n} active | Filtres, {n} actifs | new, plural (`ni18n_f`) |
| Custom period | Période personnalisée | new |
| Apply | Appliquer | new |
| Cancel | Annuler | yes, French: Annuler |
| From | De | yes, French: De |
| To | À | yes, French: À |
| Choose a date | Choisir une date | new |
| Both dates are included. Leave a field empty to leave that side open. | Les deux dates sont comprises. Laissez un champ vide pour ne pas limiter ce côté. | new |

The result count under the filter chips ("4 messages") is a plural: check
whether the catalogue has a "{n} messages" pair before adding one.

### Folder menu

| Source string (English) | Proposed French | In the catalogue |
| --- | --- | --- |
| Mark all as read | Marquer tout comme lu | new |
| Sync this folder | Synchroniser ce dossier | new |
| New subfolder… | Nouveau sous-dossier… | new |
| Rename… | Renommer… | yes, French: Renommer… |
| Delete folder | Supprimer le dossier | new |

### Conversation

| Source string (English) | Proposed French | In the catalogue |
| --- | --- | --- |
| Show message | Afficher le message | new |

### Composer

| Source string (English) | Proposed French | In the catalogue |
| --- | --- | --- |
| Save draft | Enregistrer le brouillon | new |
| Format | Format | new |
| Rich text | Texte enrichi | yes, French: Texte enrichi |
| Plain text | Texte brut | yes, French: Texte brut |
| Sign with OpenPGP | Signer avec OpenPGP | new |
| Encrypt with OpenPGP | Chiffrer avec OpenPGP | new |
| Request a read receipt | Demander un accusé de réception | yes, not translated into French |
| Check spelling | Vérifier l’orthographe | new |
| Spelling language | Langue du correcteur | new |
| Delete draft | Supprimer le brouillon | new |
| From | De | yes, French: De |
| To | À | yes, French: À |
| Cc / Bcc | Cc / Cci | new |
| Cancel | Annuler | yes, French: Annuler |
| Send | Envoyer | yes, French: Envoyer |
| Draft saved at {time} | Brouillon enregistré à {time} | new |
| Reply to {name} | Répondre à {name} | new |
| Conversation history | Historique de la conversation | new |
| Hide history | Masquer l’historique | new |
| Show previous message | Afficher le message précédent | new |
| {name} wrote, on {date}: | {name} a écrit, le {date} : | new |
| Remove {name} | Retirer {name} | yes, French: Supprimer {name} |
| Send from | Envoyer depuis | new |
| Manage accounts… | Gérer les comptes… | new |

### Tags

| Source string (English) | Proposed French | In the catalogue |
| --- | --- | --- |
| Favorite | Favori | new |
| Search or create… | Rechercher ou créer… | new |
| New tag… | Nouvelle étiquette… | new |
| Manage tags… | Gérer les étiquettes… | new |
| Tag: {name} | Étiquette : {name} | new |
| New Tag | Nouvelle étiquette | new |
| Name | Nom | yes, French: Nom |
| Color | Couleur | yes, French: Couleur |
| Preview | Aperçu | yes, French: Aperçu |
| Create | Créer | yes, French: Créer |
| Tag name | Nom de l’étiquette | new |
| Blue | Bleu | new |
| Green | Vert | new |
| Yellow | Jaune | new |
| Orange | Orange | new |
| Red | Rouge | new |
| Purple | Violet | new |
| Brown | Marron | new |
| Gray | Gris | new |
| Edit {name} | Modifier {name} | new |
| Delete {name} | Supprimer {name} | new |
| Import tags from servers… | Importer les étiquettes des serveurs… | new |
| Close | Fermer | yes, French: Fermer |

### Attachments

| Source string (English) | Proposed French | In the catalogue |
| --- | --- | --- |
| Attachments | Pièces jointes | yes, French: Pièces jointes |
| Save all… | Tout enregistrer… | new |
| Open | Ouvrir | yes, French: Ouvrir |
| Open with… | Ouvrir avec… | new |
| Save all attachments… | Enregistrer toutes les pièces jointes… | new |
| Executable file: open it only if you trust it. | Fichier exécutable : n’ouvrez que s’il est de confiance. | new |
| Large: some servers will refuse it. | Volumineux : certains serveurs le refuseront. | new |
| Send a link | Envoyer un lien | new |
| Attach to message | Joindre au message | new |
| Insert in text | Insérer dans le texte | new |
| Upload and share a link | Téléverser et partager un lien | new |
| Send without an attachment? | Envoyer sans pièce jointe ? | new |
| Your message mentions an attachment (“{word}”), but no file is attached. | Votre message mentionne une pièce jointe (« {word} »), mais aucun fichier n’y est joint. | new |
| Send Anyway | Envoyer quand même | new |
| Attach a File… | Joindre un fichier… | new |
| The server refused the message because it is too large. | Le serveur a refusé le message : il est trop volumineux. | new |

### Plurals to add

| Singular (English) | Plural (English) | French singular | French plural |
| --- | --- | --- | --- |
| {n} attachment | {n} attachments | {n} pièce jointe | {n} pièces jointes |
| {n} message | {n} messages | {n} message | {n} messages |
| {n} file | {n} files | {n} fichier | {n} fichiers |
| {n} participant | {n} participants | {n} participant | {n} participants |
