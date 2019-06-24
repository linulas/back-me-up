# Back Me Up
This scipt is used to perform backups of files or folders from the subdirectories of a specific user

## Setup
To start using, clone or download the files and go to the folder in a command line, give it permissions and run the script to start the setup:

### Unix (Mac OS & Linux)
````
cd insert_path_to_files_here
````

````
sudo chmod 777
````

````
sudo ./backup.sh
````

### Windows
To use with Windows, bash needs to be enabled by following [this tutorial](https://www.windowscentral.com/how-install-bash-shell-command-line-windows-10), or alternativley, downlad and install [git bash cmd](https://git-scm.com/downloads).

Start the windows or bass cmd as administrator and navigate to the folder where the script and other files are and launch the setup with:
````
sh ./backup.sh
````

## Usage
Backup default folder specified in the setup:
````
backup
````
Backup another folder to the default output location (path starts at home directory of the current user):
````
backup path_to_folder
````

### Flags

-u&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; --User - Specifies another user than the current one

-f&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; --File - Specify a file instead of a directory

-b&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; --Backup - Specify an alternate bakcup output location

### Examples

Backup a single file to defautl location:
````
backup -f path_to_file
````
Backup file from another user to a specific location
````
backup -u 'John Snow' -f path_to_file -b path_to_location
````

## Contact
If you have any questions, bugs to report, or suggestuions on how to improve the script, please let me know. Or fork the script and improve it yourself and feel free to add yourself as author.

contact@linusbrannstrom.dev