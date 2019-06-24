#!/bin/bash

####### Author ####### ############ Website ############ ################ Repository ###############
## Linus Brännström ## ## https://linusbrannstrom.dev ## ## https://github.com/linulas/back-me-up ##
###################### ################################# ###########################################

# This bash script is used to perform various backups

######################## Initialization ########################

# Set alias and path to script and generate config file for Linux and MacOS
function unixConfig () {
	sudo cp ./backup.sh /usr/local/bin/backup.sh;
	sudo touch ./backup.conf;
	sudo bash -c "echo user_path=$2 >> ./backup.conf";
	sudo bash -c "echo user=$(whoami) >> ./backup.conf";
	sudo bash -c "echo location=$3 >> ./backup.conf";
	sudo bash -c "echo folder=$4 >> ./backup.conf";
	if [ $1 == MacOS]; then
		sudo mv ./backup.conf /etc/defaults/backup.conf;
		sudo bash -c "echo alias backup='/usr/local/bin/backup.sh' >> ~/.bash_profile";
		. ~/.bash_profile;
	else
		sudo mv ./backup.conf /etc/default/backup.conf;
		sudo bash -c "echo alias backup='/usr/local/bin/backup.sh' >> ~/.bashrc";
		. ~/.bashrc;
	fi
}

# Set default backup output location and default folder to back up for Linux and MacOS
function unixDefault () {
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
		folder="$2"$(whoami);
	else
		folder=$res;
	fi
	while [ ! -d $folder ]; do
		printf "$folder is not a location, try again: ";
		read res;
		if [ "$res" == "" ]; then
		folder="$2"$(whoami);
		else
			folder=$res;
		fi
	done

	unixConfig $1 $2 $backup $folder;
}

# Set path to script and generate config file for Windows
function windowsConfig () {
	mkdir $1$2/Scripts;
	mkdir $1$2/Scripts/"Back Me Up";
	cp backup.sh $1$2/Scripts/"Back Me Up"/backup.sh;
	echo user_path="$1" > $1$2/Scripts/"Back Me Up"/backup.conf;
	echo user="$2" >> $1$2/Scripts/"Back Me Up"/backup.conf;
	echo location="$3" >> $1$2/Scripts/"Back Me Up"/backup.conf;
	echo folder="$4" >> $1$2/Scripts/"Back Me Up"/backup.conf;
}

# Set default backup output location and default folder to back up for Windows
windowsDefault () {
	printf "Backup output location: ";
	read res;
	if [ "$res" == "" ]; then
		mkdir C:/Program/"Back Me Up";
		backup="C:/Program/Back\ Me\ Up";
	else
		backup=$res;
	fi
	while [ ! -d $backup ]; do
		printf "$backup is not a location, try again: ";
		read res;
		if [ "$res" == "" ]; then
		    mkdir "C:/Program/Back Me Up";
			backup="C:/Program/Back\ Me\ Up";
		else
			backup=$res;
		fi
	done

	printf "Folder to be backed up: ";
	read res;
	if [ "$res" == "" ]; then
		folder=$1$(whoami);
	else
		folder=$res;
	fi
	while [ ! -d $folder ]; do
		printf "$folder is not a location, try again: ";
		read res;
		if [ "$res" == "" ]; then
			folder=$1$(whoami);
		else
			folder=$res;
		fi
	done
	
	windowsConfig "$1" $(whoami) "$backup" "$folder";
}

# Gets the currently used operating system
function getOS {
	case "$OSTYPE" in
  		solaris*) os=SOLARIS ;;
  		darwin*)  os=MacOS ;; 
  		linux*)   os=LINUX ;;
  		bsd*)     os=BSD ;;
  		msys*)    os=WINDOWS ;;
		*)        os=unknown: $OSTYPE ;;
	esac
	echo $os;
}

# Initializes the script with conf file, script location and alias
function init {

	echo "";
	echo "****************************** Back Me Up ******************************";
	echo "";
	echo "************ Enter the information, leave blank for default ************";
	echo "";

	os=$(getOS);

	echo "Operating system: $os";
	echo "";
	
	case "$os" in
		MacOS)   user_path=/Users/ ;;
		LINUX)   user_path=/home/ ;;
		WINDOWS) user_path=C:/Users/ ;;
		*)		 user_path=/home/ ;;
	esac

	if [ "$os" == "WINDOWS" ]; then
		windowsDefault "$user_path";
	else
		unixDefault "$os" "$user_path";
	fi
	echo "The backup location is: $backup";
	echo "The folder to backup is: $folder";
	echo "";
	echo "********************************* End **********************************";
	echo "";

	exit;
}

# Verifies that there is a config file, if not, initializes setup
if [ ! -f /etc/defaults/backup.conf ] && 
[ ! -f C:/Users/$(whoami)/Scripts/'Back Me Up'/backup.conf ] && 
[ ! -f /etc/default/backup.conf ]; then
	init;
fi

######################## Main Script Start ########################

# Load script info
. ./info.txt;

echo "";
echo "****************************** Back Me Up ******************************";
echo "";

echo "------------------------";
echo "Version: $version";
echo "Author: $author";
echo "------------------------";
echo "";

# Get operating system
os=$(getOS);

# Get config file
if [ "$os" == "WINDOWS" ]; then
	. c:/Users/$(whoami)/Scripts/"Back Me Up"/backup.conf;
else
	if [ "$os" == "MacOS" ]; then
		. /etc/defaults/backup.conf
	else
		. /etc/default/backup.conf
	fi
fi

# Set default values from config file
user_path="$user_path";
user="$user";
default_backup_location="$location";
default_backup_folder="$folder";
while getopts 'u:f:b:' option; do
    case "${option}" in
	u)
	    user="${OPTARG}";;
	f)
	    file="${OPTARG}";;
	b)
	    backup_location="${OPTARG}";;
	\?)
	    exit 42;;
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
function total_files {
        find $1 -type f | wc -l
}

# Reports a total number of directories for a given directory
function total_directories {
        find $1 -type d | wc -l
}

# Reports total number of directories archived
function total_archived_directories {
	if [ "$os" == "WINDOWS" ]; then
        tar -tzf $1 --force-local | grep  /$ | wc -l
	else
        sudo tar -tzf $1 | grep  /$ | wc -l
	fi
}

# Reports total number of files archived
function total_archived_files {
	if [ "$os" == "WINDOWS" ]; then
        tar -tzf $1 --force-local | grep -v /$ | wc -l
	else
        sudo tar -tzf $1 | grep -v /$ | wc -l
	fi
}

# Backs up a single file
function backup_file {
    echo "File to back up: $file"
    echo "Backing up..."
    input=$user_path$user/$file
    file_name=$(basename "$input")
    output=${path}/${user}_backup_$(date +%Y-%m-%d_%H%M%S)_${file_name}
    cp $input $output
	echo "";
	echo "********************************* End **********************************";
	echo "";
}

# Backs up a given directory
function backup_directory {
	src_files="$( total_files $input )"
	src_directories="$( total_directories $input )"

	echo "Files to be included: $src_files" | awk '$1=$1';
	echo "Directories to be included: $src_directories" | awk '$1=$1';
	echo "";
	echo "Backing up...";
	echo "";

	if [ "$os" == "WINDOWS" ]; then
		tar -czf "$output" "$input" --force-local 2> /dev/null 
	else
		sudo tar -czf "$output" "$input" 2> /dev/null 
		echo "";
	fi
	arch_files=$( total_archived_files $output )
	arch_directories=$( total_archived_directories $output )

	echo "Folders archived: $arch_directories" | awk '$1=$1';
	echo "Files archived: $arch_files" | awk '$1=$1';

	if [ $src_files -eq $arch_files ]; then
			echo "";
        	echo "Backup of $input completed."
			echo "";
        	echo "Details about the output backup file:"
        	ls -l "$output"
			echo "";
			echo "********************************* End **********************************";
			echo "";
	else
			echo "";
        	echo "Backup of $input failed"
			echo "";
			echo "********************************* End **********************************";
			echo "";
	fi
}

# Back up directory or a single file
if [ -z $file ]; then
    backup_directory
else
    backup_file
fi
