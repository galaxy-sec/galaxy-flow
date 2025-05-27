SRC=$1
USER=$2 ;
HOST=$3
DST=$4 ;
  scp -r  $SRC $USER@$HOST:$DST
  return $? ;
