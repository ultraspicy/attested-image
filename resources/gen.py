'''
Usage:
    python generate_random_list.py <length> <start_range> <end_range> <filename>
'''
import random
import argparse

def generate_random_list(length, start_range, end_range):
    return [random.randint(start_range, end_range) for _ in range(length)]

def main():
    parser = argparse.ArgumentParser(description='Generate a random comma-separated list of numbers and write to a file.')
    parser.add_argument('length', type=int, help='The length of the list.')
    parser.add_argument('start_range', type=int, help='The start of the range.')
    parser.add_argument('end_range', type=int, help='The end of the range.')
    parser.add_argument('file_name', type=str, help='The output file name.')

    args = parser.parse_args()

    random_list = generate_random_list(args.length, args.start_range, args.end_range)
    with open(args.file_name, 'w') as f:
        f.write(','.join(map(str, random_list)))

if __name__ == '__main__':
    main()

