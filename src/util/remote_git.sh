# URL="git@galaxy-sec.org:free/gxl-lab.git" ;
# REPO="gxl-lab" ;
URL=$1 ;
REPO=$2 ;
TAG=$3 ;
UPDATE=$4 ;
DST=$5
FULL_PATH=$DST/$REPO ;
if test ! -e $DST ; then
  mkdir -p $DST ;
fi
if test $UPDATE = "true" ; then
  if test  -e $DST/$REPO ; then
    rm -rf $DST/$REPO ;
  fi
fi
if test ! -e $DST/$REPO ; then
  cd $DST ;
  git clone $URL  $REPO ;
  cd $REPO ;
  git checkout  $TAG ;
fi
