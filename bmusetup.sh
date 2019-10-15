######################## Initialization ########################

# Mounts a remote folder to a local path
function cntRemoteFolder() {
    if [ "$os" == "WINDOWS" ]; then
        connection="\\\\$2\\$3 /user:$1 $5"
        net use $4 $connection >NUL
    else
        if [ "$os" == "LINUX" ]; then
            connection="//$2/$3 $4 -o user=$1,password=$5"
            echo $connection
            sudo mount -t cifs $connection
 
        else
            connection="//local;$1:$5@$2/$3"
            sudo mount -t smbfs $connection "$4"
        fi
    fi
}

function isPinged() {
    os=$(getOS)
    if [ "$os" == "WINDOWS" ]; then
        if ping $1 >NUL; then
            echo true
        else
            echo false
        fi
    else
        if ping -c 1 $1 &>/dev/null; then
            echo true
        else
            echo false
        fi
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

function setupRemoteFolder() {
    printf "Enter remote adress: "
    read res
    isPinged=$(isPinged $res)
    while ! $isPinged; do
        printf "Could not connect find remote host, try again: "
        read res
    done

    remote_adress=$res
    os=$(getOS)

    case "$os" in
    MacOS*)
        mount=/Volumes/back_me_up
        sudo mkdir $mount
        ;;
    LINUX*)
        mount=/mnt/back_me_up
        sudo mkdir $mount
        ;;
    WINDOWS*)
        mount=z:
        ;;
    esac

    printf "Username: "
    read res
    remote_user=$res
    printf "Resource: "
    read res
    remote_resource=$res
    printf "Enter you network password: "
    read res
    password=$res
    cntRemoteFolder $remote_user $remote_adress $remote_resource $mount $password
    while ! $(isMounted $mount); do
        printf "Could not mount, try again. Username: "
        read res
        remote_user=$res
        printf "Resource: "
        read res
        remote_resource=$res
        printf "Enter you network password: "
        read res
        password=$res
        cntRemoteFolder $remote_adress $remote_user $remote_resource $mount $password
    done
    if [ "$os" == "WINDOWS" ]; then
        echo mount=true >./backup.conf
        echo location="$mount" >>./backup.conf
        echo remote_adress="$remote_adress" >>./backup.conf
        echo remote_user="$remote_user" >>./backup.conf
        echo remote_resource="$remote_resource" >>./backup.conf
        echo remote_password="$password" >>./backup.conf
    else
        sudo bash -c "echo mount=true >> ./backup.conf"
        sudo bash -c "echo location=$mount >> ./backup.conf"
        sudo bash -c "echo remote_adress=$remote_adress >> ./backup.conf"
        sudo bash -c "echo remote_user=$remote_user >> ./backup.conf"
        sudo bash -c "echo remote_resource=$remote_resource >> ./backup.conf"
        sudo bash -c "echo remote_password=$password >> ./backup.conf"
    fi
}

# Set alias and path to script and generate config file for Linux and MacOS
function unixConfig() {
    . ./info.txt
    sudo cp ./backup.sh /usr/local/bin/backup.sh
    sudo cp ./bmusetup.sh /usr/local/bin/bmubackup.sh
    sudo bash -c "echo author=$author >> ./backup.conf"
    sudo bash -c "echo version=$version >> ./backup.conf"
    sudo bash -c "echo user_path=$2 >> ./backup.conf"
    sudo bash -c "echo user=$(whoami) >> ./backup.conf"
    sudo bash -c "echo folder=$3 >> ./backup.conf"
    if [ "$1" == "MacOS" ]; then
        sudo mv ./backup.conf /etc/defaults/backup.conf
        sudo bash -c "echo alias backup='/usr/local/bin/backup.sh' >> ~/.bash_profile"
        sudo bash -c "echo alias bmusetup='/usr/local/bin/bmusetup.sh' >> ~/.bash_profile"
        . ~/.bash_profile
    else
        sudo mv ./backup.conf /etc/default/backup.conf
        sudo bash -c "echo alias backup='/usr/local/bin/backup.sh' >> ~/.bashrc"
        sudo bash -c "echo alias bmusetup='/usr/local/bin/bmusetup.sh' >> ~/.bashrc"
        . ~/.bashrc
    fi
}

# Set default backup output location and default folder to back up for Linux and MacOS
function unixSetup() {
    printf "Use remote folder as backup output (y/n)? "
    read res
    res=$(echo "$res" | tr '[:upper:]' '[:lower:]')
    case "$res" in
    y) ok=true ;;
    yes) ok=true ;;
    n) ok=true ;;
    no) ok=true ;;
    *) ok=false ;;
    esac
    while ! $ok; do
        printf "Only y, yes, n or no: "
        read res
        res=$(echo "$res" | tr '[:upper:]' '[:lower:]')
        case "$res" in
        y) ok=true ;;
        yes) ok=true ;;
        n) ok=true ;;
        no) ok=true ;;
        *) ok=false ;;
        esac
    done
    if [ "$res" == "y" ] || [ "$res" == "yes" ]; then
        setupRemoteFolder $1
    else
        printf "Backup location: "
        read res
        backup=$res
        if [ "$res" == "" ]; then
            sudo mkdir /etc/back_me_up/
            backup=/etc/back_me_up/
        else
            backup=$res
        fi
        while [ ! -d $backup ]; do
            printf "$backup is not a location, try again: "
            read res
            if [ "$res" == "" ]; then
                sudo mkdir /etc/back_me_up/
                backup=/etc/back_me_up/
            else
                backup=$res
            fi
        done
        sudo bash -c "echo location=$backup >> ./backup.conf"
        sudo bash -c "echo mount=false >> ./backup.conf"
    fi

    printf "Folder to be backed up: "
    read res
    folder=$res

    if [ "$res" == "" ]; then
        folder="$2"$(whoami)
    else
        folder=$res
    fi
    while [ ! -d $folder ]; do
        printf "$folder is not a location, try again: "
        read res
        if [ "$res" == "" ]; then
            folder="$2"$(whoami)
        else
            folder=$res
        fi
    done

    echo ""

    unixConfig $1 $2 $folder
}

# Set path to script and generate config file for Windows
function windowsConfig() {
    mkdir $1$2/Scripts
    mkdir $1$2/Scripts/"Back Me Up"
    cp backup.sh $1$2/Scripts/"Back Me Up"/backup.sh
    cp bmusetup.sh $1$2/Scripts/"Back Me Up"/bmusetup.sh
    echo user_path="$1" >>./backup.conf
    echo user="$2" >>./backup.conf
    echo folder="$3" >>./backup.conf
    mv ./backup.conf $1$2/Scripts/"Back Me Up"/backup.conf
}

# Set default backup output location and default folder to back up for Windows
windowsSetup() {
    printf "Use remote folder as backup output (y/n)? "
    read res
    res=$(echo "$res" | tr '[:upper:]' '[:lower:]')
    case "$res" in
    y) ok=true ;;
    yes) ok=true ;;
    n) ok=true ;;
    no) ok=true ;;
    *) ok=false ;;
    esac
    while ! $ok; do
        printf "Only y, yes, n or no: "
        read res
        res=$(echo "$res" | tr '[:upper:]' '[:lower:]')
        case "$res" in
        y) ok=true ;;
        yes) ok=true ;;
        n) ok=true ;;
        no) ok=true ;;
        *) ok=false ;;
        esac
    done
    if [ "$res" == "y" ] || [ "$res" == "yes" ]; then
        setupRemoteFolder $1
    else
        printf "Backup output location: "
        read res
        if [ "$res" == "" ]; then
            mkdir C:/Program/"Back Me Up"
            backup="C:/Program/Back\ Me\ Up"
        else
            backup=$res
        fi
        while [ ! -d $backup ]; do
            printf "$backup is not a location, try again: "
            read res
            if [ "$res" == "" ]; then
                mkdir "C:/Program/Back Me Up"
                backup="C:/Program/Back\ Me\ Up"
            else
                backup=$res
            fi
        done
        echo mount=false >>./backup.conf
        echo location="$backup" >>./backup.conf
    fi

    printf "Folder to be backed up: "
    read res
    if [ "$res" == "" ]; then
        folder=$1$(whoami)
    else
        folder=$res
    fi
    while [ ! -d $folder ]; do
        printf "$folder is not a location, try again: "
        read res
        if [ "$res" == "" ]; then
            folder=$1$(whoami)
        else
            folder=$res
        fi
    done

    echo ""

    windowsConfig "$1" $(whoami) "$folder"
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

# Initializes the script with conf file, script location and alias
function init() {

    echo ""
    echo "****************************** Back Me Up ******************************"
    echo ""
    echo "************ Enter the information, leave blank for default ************"
    echo ""

    os=$(getOS)

    echo "Operating system: $os"
    echo ""

    case "$os" in
    MacOS) user_path=/Users/ ;;
    LINUX) user_path=/home/ ;;
    WINDOWS) user_path=C:/Users/ ;;
    *) user_path=/home/ ;;
    esac

    if [ "$os" == "WINDOWS" ]; then
        windowsSetup "$user_path"
    else
        unixSetup "$os" "$user_path"
    fi
    echo "The backup location is: $location"
    echo "The folder to backup is: $folder"
    echo ""
    echo "********************************* End **********************************"
    echo ""

    exit
}
init
