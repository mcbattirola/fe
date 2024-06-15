import sys
import configparser
import subprocess

def main():
    if len(sys.argv) < 2:
        print("Usage: python open_url.py <path_to_url_file>")
        return

    url_file_path = sys.argv[1]

    # Read the .url file
    config = configparser.ConfigParser()
    config.read(url_file_path)

    if 'InternetShortcut' in config and 'URL' in config['InternetShortcut']:
        url = config['InternetShortcut']['URL']
        print(f"Opening URL: {url}")

        # Spawn Brave with the URL
        try:
            subprocess.run(["brave", url], check=True)
        except Exception as e:
            print(f"Failed to open URL with Brave: {e}")
    else:
        print("Invalid .url file")

if __name__ == "__main__":
    main()
