# Popovers live in windows of their own that a root grab leaves out: grab
# each visible "Tegami Mail" window but the main one, with its position.
rm -f /shots/grab/$NAME.pops
best=0; main=
for w in $(xdotool search --onlyvisible --name '^Tegami Mail$'); do
  a=$(xwininfo -id $w | awk '/Width:/{w=$NF}/Height:/{h=$NF}END{print w*h}')
  [ $a -gt $best ] && { best=$a; main=$w; }
done
n=0
for w in $(xdotool search --onlyvisible --name '^Tegami Mail$'); do
  [ "$w" = "$main" ] && continue
  geo=$(xwininfo -id $w | awk '/Absolute upper-left X/{x=$NF}/Absolute upper-left Y/{y=$NF}/Width:/{w=$NF}/Height:/{h=$NF}END{print x" "y" "w" "h}')
  set -- $geo; [ "$3" -le 1 ] && continue
  n=$((n+1)); xwd -id $w -silent | convert xwd:- /shots/grab/$NAME-pop$n.png
  echo "$NAME-pop$n.png $geo" >> /shots/grab/$NAME.pops
done
