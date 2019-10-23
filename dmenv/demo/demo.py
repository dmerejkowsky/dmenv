from path import Path
import sys

def main():
    print(sys.argv)
    print("Running demo from", Path.getcwd())


if __name__ == "__main__":
    main()
