import pikaxe
import sys

def main(args: list[str]):
    ark_path = args[0]
    print(f'Opening ark from \'{ark_path}\'')

    ark = pikaxe.Ark.from_file_path(ark_path)
    ark_entries = ark.entries

    print(f'Ark: (version = {ark.version}, encryption = {ark.encryption})')
    print(f'Found {len(ark_entries)} entries')

    for entry in ark_entries:
        print(entry.path)

if __name__ == '__main__':
    main(sys.argv[1:])