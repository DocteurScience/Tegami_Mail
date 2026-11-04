# Captures de l'application face aux maquettes

Une capture par maquette de `data/repo/design/` (même nom de fichier),
prise le 2026-11-04 sur `main` (`6d1b973`), après les corrections du retour
de design du même jour, en mode démo et en Adwaita Sans 11.

- `1x/` : aux dimensions des maquettes, pour les comparer directement.
- `2x/` : les originaux, en 2x.

Refaire la série : `docker exec tgu bash /mockup-shots.sh`, puis
`tools/dev/mockup-crop.py` (voir CLAUDE.md).

Limites : le contenu est celui du mode démo ; sans mise en scène, la
première ligne est sélectionnée ; les popovers n'ont ni coins arrondis ni
ombre (pas de compositeur) ; 15 est à la largeur minimale de l'application
(904 px, application de bureau seulement, décision du propriétaire) ; 17
utilise la taille de texte de l'application à 150 %, ce que fait le grand
texte de GNOME au travers de gtk-xft-dpi.
