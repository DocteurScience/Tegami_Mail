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

### Preferences

Screens 22 to 48. Many of these are already in the catalogue under the same or
a close wording: look each one up first and reuse its translation. Labels are
in sentence case (CLAUDE.md). Rows that explain a default keep the subtitle
short. The French is a proposal.

| Source string (English) | Proposed French |
| --- | --- |
| Preferences | Préférences |
| Accounts | Comptes |
| Messages | Messages |
| Notifications | Notifications |
| Privacy | Confidentialité |
| Application | Application |
| Search preferences | Rechercher dans les préférences |
| Reading | Lecture |
| Mark as read | Marquer comme lu |
| When a message is opened. | Quand un message est ouvert. |
| Newest message first | Message le plus récent en premier |
| In a conversation. | Dans une conversation. |
| Confirm deleting a conversation | Confirmer la suppression d’une conversation |
| Swipe to archive or delete | Balayer pour archiver ou supprimer |
| For touch screens and touchpads. | Pour les écrans tactiles et les pavés tactiles. |
| Composing | Rédaction |
| Send new messages from | Envoyer les nouveaux messages depuis |
| Replies are sent from the address that received the message. | Les réponses partent de l’adresse qui a reçu le message. |
| Last used account | Dernier compte utilisé |
| Format of new messages | Format des nouveaux messages |
| Rich text | Texte enrichi |
| Plain text | Texte brut |
| Undo send | Annuler l’envoi |
| How long a message can still be recalled. | Délai pendant lequel l’envoi peut être annulé. |
| Check spelling | Vérifier l’orthographe |
| Languages follow the system. | Les langues sont celles du système. |
| New messages | Nouveaux messages |
| Desktop notifications | Notifications de bureau |
| While the window is in the background. | Quand la fenêtre est en arrière-plan. |
| Show sender and subject | Afficher l’expéditeur et l’objet |
| Off, the notification only says “New message”. | Désactivé, la notification dit seulement « Nouveau message ». |
| Check for new messages | Vérifier les nouveaux messages |
| Automatically | Automatiquement |
| Sound and banners | Son et bannières |
| Open the system notification settings | Ouvrir les paramètres de notifications du système |
| Sound, banners and Do Not Disturb are set there. | Le son, les bannières et le mode Ne pas déranger se règlent là. |
| Remote content | Contenu distant |
| Images and remote content | Images et contenu distant |
| Remote content can tell the sender when and from where you read. | Un contenu distant peut indiquer à l’expéditeur quand et d’où vous lisez. |
| Never | Jamais |
| Ask | Demander |
| Always | Toujours |
| Sender photos and logos | Photos et logos des expéditeurs |
| Gravatar and website logos: the sender’s address is sent to these services. | Gravatar et logos de sites : l’adresse de l’expéditeur est envoyée à ces services. |
| Read receipts | Accusés de réception |
| Answer read-receipt requests | Répondre aux demandes d’accusé de réception |
| A receipt tells the sender you opened the message. | Un accusé indique à l’expéditeur que vous avez ouvert le message. |
| Appearance | Apparence |
| Style | Style |
| Follows the system by default. The accent colour also comes from the system. | Suit le système par défaut. L’accent vient aussi du système. |
| Follow the system | Suivre le système |
| Light | Clair |
| Dark | Sombre |
| Background | Arrière-plan |
| Keep running in the background | Continuer en arrière-plan |
| Closing the window keeps Tegami Mail running for notifications. | Fermer la fenêtre garde Tegami Mail actif pour les notifications. |
| Start at login | Lancer à l’ouverture de session |
| Keyboard and storage | Clavier et stockage |
| Single-key shortcuts | Raccourcis à une touche |
| Ctrl+? shows them all. | Ctrl+? les affiche tous. |
| Keep downloaded mail | Conserver le courrier téléchargé |
| On this computer, for reading offline. | Sur cet ordinateur, pour la lecture hors ligne. |
| Large attachments | Pièces jointes volumineuses |
| Mail accounts | Comptes de messagerie |
| Drag an account to change its order in the sidebar. | Glissez un compte pour changer son ordre dans la barre latérale. |
| On this computer | Sur cet ordinateur |
| Mail saved in local folders | Courrier enregistré dans des dossiers locaux |
| Add account… | Ajouter un compte… |
| E-mail account… | Compte e-mail… |
| GNOME Online Account… | Compte en ligne GNOME… |
| Local mail on this computer | Courrier local sur cet ordinateur |
| Import from Thunderbird or Evolution… | Importer depuis Thunderbird ou Evolution… |
| Organize mail | Organiser le courrier |
| Services | Services |
| Smart folders | Dossiers intelligents |
| Senders | Expéditeurs |
| Allowed and blocked | Autorisés et bloqués |
| OpenPGP keys | Clés OpenPGP |
| Cloud storage | Stockage en ligne |
| Translation | Traduction |
| Address books | Carnets d’adresses |
| Backup and restore | Sauvegarde et restauration |
| Display name | Nom affiché |
| Email address | Adresse e-mail |
| Account colour | Couleur du compte |
| Accent of this account’s badges and outlines. | Accent des pastilles et des contours de ce compte. |
| Show in combined views | Afficher dans les vues combinées |
| Favorites, tags and smart folders from all accounts. | Favoris, étiquettes et dossiers intelligents de tous les comptes. |
| Servers and connection | Serveurs et connexion |
| Synchronisation | Synchronisation |
| Folders | Dossiers |
| Send-as aliases | Alias d’envoi |
| Signatures | Signatures |
| OpenPGP encryption | Chiffrement OpenPGP |
| Remove account | Supprimer le compte |
| Also removes its saved password. Mail on the server is not touched. | Supprime aussi son mot de passe enregistré. Le courrier sur le serveur n’est pas touché. |
| Incoming | Réception |
| Protocol | Protocole |
| Server | Serveur |
| Port | Port |
| User name | Nom d’utilisateur |
| Password | Mot de passe |
| The password is kept in the system keyring, never in plain text. | Le mot de passe est conservé dans le trousseau du système, jamais en clair. |
| Outgoing (SMTP) | Envoi (SMTP) |
| Use other credentials | Utiliser d’autres identifiants |
| Off: the incoming credentials are used. | Désactivé : ceux de la réception sont utilisés. |
| Advanced | Avancé |
| API token, custom OAuth app, certificate for another name | Jeton d’API, application OAuth personnalisée, certificat d’un autre nom |
| Test connection | Tester la connexion |
| Connected: receiving and sending work. | Connecté : réception et envoi fonctionnent. |
| Instant new mail (push) | Courrier instantané (push) |
| Turn off for servers that block push connections. | Désactivez-le pour les serveurs qui bloquent les connexions push. |
| Automatic cleanup | Nettoyage automatique |
| Empty Junk | Vider les indésirables |
| Empty Trash | Vider la corbeille |
| After 30 days | Après 30 jours |
| As the application | Comme l’application |
| Leaves out Trash, Junk and Drafts. | Hors corbeille, indésirables et brouillons. |
| Special folders | Dossiers spéciaux |
| Sent | Envoyés |
| Drafts | Brouillons |
| Archive | Archives |
| Trash | Corbeille |
| Junk | Indésirables |
| Automatic | Automatique |
| Follows the server’s own markings unless you pick a folder. Nothing already on the server moves. | Suit les repères du serveur, sauf si vous choisissez un dossier. Rien de ce qui est déjà sur le serveur ne bouge. |
| Folder order | Ordre des dossiers |
| Hidden folders | Dossiers masqués |
| No hidden folders. | Aucun dossier masqué. |
| Add alias… | Ajouter un alias… |
| Sends through this account | Envoi via ce compte |
| Add signature | Ajouter une signature |
| Import file… | Importer un fichier… |
| Start new messages with this signature | Commencer les nouveaux messages avec cette signature |
| Edit HTML | Modifier le HTML |
| Delete signature | Supprimer la signature |
| Key | Clé |
| Sign messages by default | Signer les messages par défaut |
| Hide the subject of encrypted mail | Masquer l’objet des messages chiffrés |
| Manage keys | Gérer les clés |
| Add tag… | Ajouter une étiquette… |
| Find tags in other clients | Chercher les étiquettes d’autres clients |
| Edit tag | Modifier l’étiquette |
| Name | Nom |
| Colour | Couleur |
| Custom colour… | Couleur personnalisée… |
| IMAP keyword | Mot-clé IMAP |
| Delete tag | Supprimer l’étiquette |
| Add filter… | Ajouter un filtre… |
| Apply now | Appliquer maintenant |
| Edit filter | Modifier le filtre |
| Name (optional) | Nom (facultatif) |
| If | Si |
| Conditions | Critères |
| All must match | Tous doivent correspondre |
| Add condition | Ajouter un critère |
| Then | Alors |
| Add action | Ajouter une action |
| Delete filter | Supprimer le filtre |
| Add smart folder… | Ajouter un dossier intelligent… |
| Edit smart folder | Modifier le dossier intelligent |
| Criteria | Critères |
| Delete smart folder | Supprimer le dossier intelligent |
| Search senders | Rechercher un expéditeur |
| Allowed senders | Expéditeurs autorisés |
| Blocked senders | Expéditeurs bloqués |
| Your keys | Vos clés |
| Correspondents’ keys | Clés des correspondants |
| Import key… | Importer une clé… |
| Generate key… | Générer une clé… |
| Fetch by address… | Chercher par adresse… |
| Trust… | Approuver… |
| Trusted | Approuvée |
| Cloud account | Compte de stockage |
| Service | Service |
| Upload folder | Dossier d’envoi |
| Links | Liens |
| Links expire after | Les liens expirent après |
| Protect links with a password | Protéger les liens par un mot de passe |
| Offer this account in the composer | Proposer ce compte dans le composeur |
| Check connection | Vérifier la connexion |
| Translate into | Traduire vers |
| Check the service | Vérifier le service |
| Address book | Carnet d’adresses |
| New contacts | Nouveaux contacts |
| Save new contacts in | Enregistrer les nouveaux contacts dans |
| Add address book… | Ajouter un carnet d’adresses… |
| Open Contacts | Ouvrir Contacts |
| Open Online Accounts | Ouvrir Comptes en ligne |
| Export settings | Exporter les réglages |
| Import settings… | Importer des réglages… |
| Replaces your settings. Keep your accounts in place: removing them first deletes their passwords. | Remplace vos réglages. Gardez vos comptes en place : les retirer avant supprime leurs mots de passe. |

Plurals for the list subtitles: "{n} tags", "{n} filters", "{n} smart folders",
"{n} address books", "{n} signatures", "{n} addresses" (alias count).

### Plurals to add

| Singular (English) | Plural (English) | French singular | French plural |
| --- | --- | --- | --- |
| {n} attachment | {n} attachments | {n} pièce jointe | {n} pièces jointes |
| {n} message | {n} messages | {n} message | {n} messages |
| {n} file | {n} files | {n} fichier | {n} fichiers |
| {n} participant | {n} participants | {n} participant | {n} participants |
