#!/bin/bash

############Author############ ############ Website ############
###### Linus Brännström ###### ###### linusbrannstrom.dev ######
############################## #################################



function init {

	echo "";
	echo "****************************** Back Me Up ******************************";
	echo "";
	echo "************ Enter the information, leave blank for default ************";
	echo "";

	case "$OSTYPE" in
  		solaris*) os=SOLARIS ;;
  		darwin*)  os=MacOS ;; 
  		linux*)   os=LINUX ;;
  		bsd*)     os=BSD ;;
  		msys*)    os=WINDOWS ;;
		*)        os=unknown: $OSTYPE ;;
	esac

	echo "Operating system: $os";
	echo "";

	case "$os" in
		MacOS)   user_path=/Users/ ;;
		LINUX)   user_path=/home/ ;;
		WINDOWS) user_path=/C:/Users ;;
		*)		 user_path=/home/ ;;
	esac

	echo "Path to user folder: $user_path";
	echo "";

	printf "Backup location: ";
	read res;
	backup=$res;
	if [ "$res" == "" ]; then
		sudo mkdir /etc/back_me_up/;
		backup=/etc/back_me_up/;
	else
		backup=$res;
	fi
	while [ ! -d $backup ]; do
		printf "$backup is not a location, try again: ";
		read res;
		if [ "$res" == "" ]; then
		    sudo mkdir /etc/back_me_up/;
			backup=/etc/back_me_up/;
		else
			backup=$res;
		fi
	done

	printf "Folder to be backed up: ";
	read res;
	folder=$res;

	if [ "$res" == "" ]; then
		folder="$user_path"$(whoami);
	else
		folder=$res;
	fi
	while [ ! -d $folder ]; do
		printf "$folder is not a location, try again: ";
		read res;
		if [ "$res" == "" ]; then
		folder="$user_path"$(whoami);
		else
			folder=$res;
		fi
	done

	echo "The backup location is: $backup";
	echo "The folder to backup is: $folder";

	sudo cp ./backup.sh /usr/local/bin/backup.sh;
	sudo touch ./backup.conf;
	sudo bash -c "echo user=$(whoami) >> ./backup.conf";
	sudo bash -c "echo location=$backup >> ./backup.conf";
	sudo bash -c "echo folder=$folder >> ./backup.conf";
	sudo mv ./backup.conf /etc/defaults/backup.conf;

	exit;
}

if [ ! -f /etc/back_me_up.conf ]; then
	init;
fi

# This bash script is used to perform various backups

. /etc/defaults/back_me_up.conf;
user="$user";
default_backup_location="$location";
default_backup_folder="$folder";

while getopts 'u:f:b:' option; do
    case "${option}" in
	u)
	    user="${OPTARG}";;
	f)
	    file="${OPTARG}";;
	l)
	    backup_location="${OPTARG}";;
	\?)
	    exit 42;;
    esac
done

# shift opts away
shift $((OPTIND - 1))

if [ -z $backup_location ]; then
	path="$default_backup_location"
else
	path=$backup_location
fi

if [ -z $file ]; then
    if [ -z $1 ]; then
        input=/home/$user
        output=${path}/${user}_"Home"_$(date +%Y-%m-%d_%H%M%S).tar.gz
    else
        if [ ! -d "/home/$user/$1" ]; then
                echo "Requested $1 directory doesn't exist."
                exit 1
        fi
        input=/home/$user/$1
        output=${path}/${user}_${1}_$(date +%Y-%m-%d_%H%M%S).tar.gz
    fi
fi

# Reports a total number of files for a given directory.
function total_files {
        find $1 -type f | wc -l
}

# Reports a total number of directories for a given directory
function total_directories {
        find $1 -type d | wc -l
}

# Reports total number of directories archived
function total_archived_directories {
        tar -tzf $1 | grep  /$ | wc -l
}

# Reports total number of files archived
function total_archived_files {
        tar -tzf $1 | grep -v /$ | wc -l
}

function backup_file {
    echo "File to back up: $file"
    echo "Backing up..."
    input=/home/$user/$file
    echo "input: $input"
    file_name=$(basename $input)
    output=${path}/${user}_backup_$(date +%Y-%m-%d_%H%M%S)_${file_name}
    sudo cp $input $output
}

function backup_directory {
	src_files=$( total_files $input )
	src_directories=$( total_directories $input )

	echo "Files to be included: $src_files"
	echo "Directories to be included: $src_directories"
	echo "Backing up..."

	sudo tar -czf $output $input 2> /dev/null
	arch_files=$( total_archived_files $output )
	arch_directories=$( total_archived_directories $output )

	echo "Folders archived: $arch_directories"
	echo "Files archived: $arch_files"

	if [ $src_files -eq $arch_files ]; then
        	echo "Backup of $input completed."
        	echo "Details about the output backup file:"
        	ls -l $output
	else
        	echo "Backup of $input failed"
	fi
}

if [ -z $file ]; then
    backup_directory
else
    backup_file