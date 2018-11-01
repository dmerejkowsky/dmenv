import ssl
import urllib.request
import sys
import os
import shutil


def show_progress(xfered, size):
    percent = float(xfered) / size * 100
    print("Downloading: %.0f%%" % percent, flush=True, end="\r")


def download(url, output_file):
    context = ssl.create_default_context()
    url_obj = urllib.request.urlopen(url, context=context)

    content_length = url_obj.headers.get("content-length")
    size = int(content_length)
    buff_size = 100 * 1024
    xferd = 0

    dest_file = open(output_file, "wb")
    try:
        while True < size:
            data = url_obj.read(buff_size)
            if not data:
                break
            xferd += len(data)
            show_progress(xferd, size)
            dest_file.write(data)
        if xferd != size:
            # short read :/
            sys.exit("Error: expecting {}, got {}".format(xferd, size))
    finally:
        dest_file.close()


def move_in_path(dmenv_bin):
    entries = os.environ.get("PATH").split(os.path.pathsep)
    print("Heres are the possible locations to install dmenv")
    print("Select one element in the list")
    for i, entry in enumerate(entries, start=1):
        print("%2d" % i, entry)
    answer = input("> ")
    entry = None
    while True:
        try:
            num = int(answer)
            entry = entries[num - 1]
            break
        except ValueError:
            print("Please enter a number")
        except IndexError:
            print("Please choose between 0 and", len(entries))
    print(dmenv_bin, "->", entry)
    dest = os.path.join(entry, dmenv_bin)
    if os.path.exists(dest):
        if "--upgrade" in sys.argv:
            os.remove(dest)
        else:
            sys.exit("Error: %s already exists. Use --upgrade to upgrade" % dest)
    shutil.move(dmenv_bin, entry)
    os.chmod(dest, 0o755)


def main():
    url = "https://dmerej.info/pub/dmenv-%s" % sys.platform
    if sys.platform == "windows":
        url += ".exe"
        out = "dmenv.exe"
    else:
        out = "dmenv"

    if os.path.exists(out):
        print(out, "already exists, skipping download")
    else:
        print("Downloading", url, "to", out)
        download(url, out)
    move_in_path(out)


if __name__ == "__main__":
    main()
