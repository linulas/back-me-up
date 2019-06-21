#!/bin/bash

# This bash script is used to perform various backups

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

# shift opts away
shift $((OPTIND - 1))

if [ -z $user ]; then
	user=$(whoami)
fi

if [ -z $backup_location ]; then
	if grep -qs '/mnt/backups ' /proc/mounts; then
        	echo "Remote backup location is mounted."
	else
        	echo "Remote backup location is not mounted, mounting...."
        	sudo mount -t cifs //192.168.0.144/Backups/Raspberrypi /mnt/backups --verbose -o user=Linus
	fi
	path="/mnt/backups"
else
	path=$backup_location
fi

if [ -z $file ]; then
    if [ -z $1 ]; then
        input=/home/$user
        output=${path}/${user}_"Home_"$(date +%Y-%m-%d_%H%M%S).tar.gz
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

delold=$1

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
        	if [ -z $backup_location ]; then
                	echo "Deleting old backups..."
			echo "Test: $delold"
                	if [ -z $delold ]; then
				echo "no args"
                        	/home/pi/Scripts/delold.sh
                	else
				echo "Sending args: $delold"
                        	/home/pi/Scripts/delold.sh $delold
                	fi
        	fi
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
fi
