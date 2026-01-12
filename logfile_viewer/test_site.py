from server import Server
from log import Log

import pytest
import threading
import time
import requests
from lorem_text import lorem
from pathlib import Path
import shutil
import gzip
import os
import json

@pytest.fixture(scope="session")
def make_logs():
    """
    Creates basic log files for testing
    """

    test_file_dir = Path("./pytest_test_files")
    if test_file_dir.exists():
        shutil.rmtree(test_file_dir)
    test_file_dir.mkdir()
    test_file_subdir = test_file_dir / "inside"
    test_file_subdir.mkdir()
    for i in range(1, 5):
        with open((test_file_subdir / str(i)), 'w') as file:
            file.write("")


    logs = {
        "basic": {"logfile": f"{test_file_dir}/basic.txt"},
        "compressed": {"logfile": f"{test_file_dir}/compressed.gz"},
        "break_char": {"logfile": f"{test_file_dir}/break_char.txt", "breaksymbol": "---"},
        "directory": {"logfile": f"{test_file_dir}"}
    }

    formatted_logs = {}
    for key, value in logs.items():
        log = Log(value)
        formatted_logs[key] = log

    for key, value in logs.items():
        if key == "directory":
            continue
        text = "STARTFILE\n"
        for i in range(10):
            text += lorem.words(6) + "\n"
            if "breaksymbol" in value:
                text += lorem.words(4) + "\n"
                text += value["breaksymbol"] + "\n"
        text += "ENDFILE"

        if ".gz" in value["logfile"]:
            with gzip.open(value["logfile"], 'w') as file:
                content = file.write(text.encode('utf-8'))
        else:
            with open(value["logfile"], 'w') as file:
                file.write(text)
        
    yield formatted_logs

    try:
        shutil.rmtree(test_file_dir)
    except Exception:
        pass

def make_request(address):
    try:
        response = requests.get(f"http://{address}", timeout=3)        
        assert response.ok
        return response
    except requests.exceptions.ConnectionError:
        print(f"Request made to \'http://{address}\' and failed")
        pytest.fail(f"Failed to connect to the server. Is it running?")

def make_api_request(address):
    try:
        response_string = make_request(address).text
        return json.loads(response_string)
    except json.JSONDecodeError:
        pytest.fail("Failed to decode returned json")

def check_list_order(list, substrings):
    previous_index = -1
    for substring in substrings:
        index = next(i for i, item in enumerate(list) if substring in item)
        assert index > previous_index
        previous_index = index

@pytest.fixture(scope="session")
def server(make_logs):
    stop_server_flag = threading.Event()
    port = 8900#random.randint(10000, 60000)
    logs = make_logs
    host_address = ("127.0.0.1", port)
    host_address_string = f"{host_address[0]}:{host_address[1]}"
    server = Server(host_address=host_address, logs=logs)
    thread = threading.Thread(target=server.run, args=(stop_server_flag,), daemon=True)
    thread.start()
    time.sleep(1)
    yield host_address_string, logs
    stop_server_flag.set()
    thread.join(timeout=5)

def test_index_page(server):
    """Tests that the server's home page returns a 200 OK status."""
    host_address = server[0]
    logs = server[1]

    response = make_request(host_address)

    assert response.status_code == 200
    
    assert "Welcome To LogFileViewer" in response.text

    for keyword in logs:
        assert keyword in response.text

def test_basic_log(server):
    host_address = server[0]
    request_address = f"{host_address}/api/log/basic"
    
    response = make_api_request(request_address)

    sub_strings_in_order = [
        "ENDFILE",
        "STARTFILE",
    ]
    check_list_order(response["contents"], sub_strings_in_order)


def test_compressed_log(server):
    host_address = server[0]
    request_address = f"{host_address}/api/log/compressed"

    response = make_api_request(request_address)

    sub_strings_in_order = [
        "ENDFILE",
        "STARTFILE",
    ]
    check_list_order(response["contents"], sub_strings_in_order)

def test_break_char(server):
    host_address = server[0]
    request_address = f"{host_address}/api/log/break_char"

    response = make_api_request(request_address)

    sub_strings_in_order = [
        "ENDFILE",
        "STARTFILE",
    ]
    check_list_order(response["contents"], sub_strings_in_order)

    for line in response["contents"]:
        if "STARTFILE" in line or "ENDFILE" in line:
            continue
        if r"\n" not in line and r"<br>" not in line: #Log lines should be over multiple lines
            pytest.fail("Failed to find required substrings in logs")


def test_directory(server):
    host_address = server[0]
    log = server[1]["directory"]
    request_address = f"{host_address}/log/directory"

    response = make_request(request_address)
    
    directory_contents = os.listdir(log.log_file)

    for directory_item in directory_contents:
        assert directory_item in response.text

def test_directory(server):
    host_address = server[0]
    log = server[1]["directory"]
    request_address = f"{host_address}/log/directory/inside"

    response = make_request(request_address)
    
    directory_contents = os.listdir(log.log_file + "/inside")

    for directory_item in directory_contents:
        assert directory_item in response.text

