import sys
import json
import csv
import click
import requests


def read_json(file_path):
    with open(file_path, 'r') as f:
        return json.load(f)


def check_row(row, columns):
    for column in columns:
        if row.get(column) is None:
            print("Error: %s column must be present in csv file." % column)
            sys.exit(1)


def csv_to_json(csvFilePath, check_row, transform_row):
    jsonArray = []

    # read csv file
    with open(csvFilePath, encoding='utf-8') as csvf:
        # load csv file data using csv library's dictionary reader
        csvReader = csv.DictReader(csvf)

        # convert each csv row into python dict
        for index, row in enumerate(csvReader):
            check_row(row)
            # add this python dict to json array
            row = transform_row(row, index)
            jsonArray.append(row)
        return jsonArray


def transform_row4file(row, idx):
    try:
        row['size'] = int(row.get('size'))
        return row
    except Exception:
        print("Invalid row: %s, size must be integer" % str(idx))
        sys.exit(1)


def batch_import_files(data):
    success = []
    failed = []
    for item in data:
        r = requests.post('http://localhost:3000/api/v1/files', json=item)
        if r.status_code == 201:
            print('Successfully imported file: ' + item['filename'])
            success.append(r.json())
        else:
            print('Failed to import file: ' + item['filename'])
            failed.append({'filename': item['filename'], 'error': r.text})
    return {
        'success': success,
        'failed': failed,
        'data': data
    }


columns = {
    'file': ['size', 'filename', 'uploader', 'hash']
}

transform_fns = {
    'file': transform_row4file
}


@click.command()
@click.option('--json', help='Path to json file.')
@click.option('--csv', help='Path to csv file.')
@click.option('--output', help='Path to output file.')
def batch_import_file(json, csv, output):
    mode = 'file'
    json_data = []
    if json:
        json_data = read_json(json)

    if csv:
        json_data = csv_to_json(
            csv,
            lambda row: check_row(row, columns.get(mode)),
            transform_fns.get(mode)
        )

    output = 'output.json' if not output else output

    with open(output, 'w') as f:
        json.dump(batch_import_files(json_data), f, indent=4)
