#!/bin/bash

####### Author ####### ############ Website ############ ################ Repository ###############
## Linus Brännström ## ## https://linusbrannstrom.dev ## ## https://github.com/linulas/back-me-up ##
###################### ################################# ###########################################

# This bash script is used to perform various backups

# Verifies that there is a config file
function cfgExists() {
	if [ ! -f /etc/defaults/backup.conf ] &&
		[ ! -f C:/Users/$(whoami)/Scripts/'Back Me Up'/backup.conf ] &&
		[ ! -f /etc/default/backup.conf ]; then
		echo false
	else
		echo true
	fi
}

function isMounted() {
	if [ $(getOS) == "WINDOWS" ]; then
		if [ ! -d $1 ]; then
			echo true
		else
			echo false
		fi
	else
		if mount | grep $1 >/dev/null; then
			echo true
		else
			echo false
		fi

	fi
}

# Mounts a remote folder to a local path
function cntRemoteFolder() {
	if [ "$os" == "WINDOWS" ]; then
		connection="\\\\$2\\$3 /user:$1 $5"
		net use $4 $connection >NUL
	else
		connection="//local;$1:$5@$2/$3"
		sudo mount -t smbfs $connection "$4"
	fi
}

function automateBackup() {
	if [ "$os" = "WINDOWS" ]; then
		echo " Windows: $1"
	else
		sudo watch -n $1 /usr/local/bin/backup.sh $2 $3 $4 $5 $6 $7
	fi
}

# Gets the currently used operating system
function getOS() {
	case "$OSTYPE" in
	solaris*) os=SOLARIS ;;
	darwin*) os=MacOS ;;
	linux*) os=LINUX ;;
	bsd*) os=BSD ;;
	msys*) os=WINDOWS ;;
	*) os=unknown: $OSTYPE ;;
	esac
	echo $os
}

######################## Main Script Start ########################

# Init setup if there's no config file
if [ ! $(cfgExists) == true ]; then
	./bmusetup.sh
	exit
fi
# Load script info
. ./info.txt

echo ""
echo "****************************** Back Me Up ******************************"
echo ""

echo "------------------------"
echo "Version: $version"
echo "Author: $author"
echo "------------------------"
echo ""

# Get operating system
os=$(getOS)
echo "Operating System: $os"
echo ""

# Get config file
if [ "$os" == "WINDOWS" ]; then
	. c:/Users/$(whoami)/Scripts/"Back Me Up"/backup.conf
else
	if [ "$os" == "MacOS" ]; then
		. /etc/defaults/backup.conf
	else
		. /etc/default/backup.conf
	fi
fi

# Set default values from config file
mount="$mount"
user_path="$user_path"
user="$user"
default_backup_folder="$folder"
default_backup_location="$location"
remote_adress="$remote_adress"
remote_user="$remote_user"
remote_resource="$remote_resource"
remote_password="$remote_password"

if [ "$mount" == "true" ]; then
	if ! $(isMounted $location); then
		sudo mkdir $folder
		cntRemoteFolder $remote_user $remote_adress $remote_resource $location $remote_password
	fi
fi

while getopts 'a:u:f:b:r:' option; do
	case "${option}" in
	a)
		seconds="${OPTARG}"
		;;
	u)
		user="${OPTARG}"
		;;
	f)
		file="${OPTARG}"
		;;
	b)
		backup_location="${OPTARG}"
		alternate_location="${OPTARG}"
		;;
	r)
		remote_location="${OPTARG}"
		;;
	\?)
		exit 42
		;;
	esac
done

# Shift opts away
shift $((OPTIND - 1))

if [ -z $backup_location ]; then
	path="$default_backup_location"
else
	path="$backup_location"
fi

# Set input and output
if [ -z $file ]; then
	if [ -z $1 ]; then
		input="$default_backup_folder"
		output="${path}/${user}_default_$(date +%Y-%m-%d_%H%M%S).tar.gz"
	else
		if [ ! -d "$user_path$user/$1" ]; then
			echo "Requested $1 directory doesn't exist."
			exit 1
		fi
		input=$user_path$user/$1
		output=${path}/${user}_${1}_$(date +%Y-%m-%d_%H%M%S).tar.gz
	fi
fi

# Reports a total number of files for a given directory.
function total_files() {
	find $1 -type f | wc -l
}

# Reports a total number of directories for a given directory
function total_directories() {
	find $1 -type d | wc -l
}

# Reports total number of directories archived
function total_archived_directories() {
	if [ "$os" == "WINDOWS" ]; then
		tar -tzf $1 --force-local | grep /$ | wc -l
	else
		sudo tar -tzf $1 | grep /$ | wc -l
	fi
}

# Reports total number of files archived
function total_archived_files() {
	if [ "$os" == "WINDOWS" ]; then
		tar -tzf $1 --force-local | grep -v /$ | wc -l
	else
		sudo tar -tzf $1 | grep -v /$ | wc -l
	fi
}

# Backs up a single file
function backup_file() {
	echo "File to back up: $file"
	echo "Backing up..."
	input=$user_path$user/$file
	file_name=$(basename "$input")
	output=${path}/${user}_backup_$(date +%Y-%m-%d_%H%M%S)_${file_name}
	sudo cp $input $output
	echo ""
	echo "********************************* End **********************************"
	echo ""
}

# Backs up a given directory
function backup_directory() {
	src_files="$(total_files $input)"
	src_directories="$(total_directories $input)"

	echo "Files to be included: $src_files" | awk '$1=$1'
	echo "Directories to be included: $src_directories" | awk '$1=$1'
	echo ""
	echo "Backing up..."
	echo ""

	if [ "$os" == "WINDOWS" ]; then
		tar -czf "$output" "$input" --force-local 2>/dev/null
	else
		sudo tar -czf "$output" "$input" 2>/dev/null
		echo ""
	fi
	arch_files=$(total_archived_files $output)
	arch_directories=$(total_archived_directories $output)

	echo "Folders archived: $arch_directories" | awk '$1=$1'
	echo "Files archived: $arch_files" | awk '$1=$1'

	if [ $src_files -eq $arch_files ]; then
		echo ""
		echo "Backup of $input completed."
		echo ""
		echo "Details about the output backup file:"
		ls -l "$output"
		echo ""
		echo "********************************* End **********************************"
		echo ""
	else
		echo ""
		echo "Backup of $input failed"
		echo ""
		echo "********************************* End **********************************"
		echo ""
	fi
}

# Repeats the backup every x seconds
if [ $seconds ]; then
	if [ ! -z $file ]; then
		automated_file="-f $file"
	fi
	if [ ! -z $alternate_location ]; then
		automated_location="-b $alternate_location"
	fi
	automateBackup $seconds $1 $automated_file $automated_location
fi

# Back up directory or a single file
if [ -z $file ]; then
	backup_directory
else
	backup_file
fi