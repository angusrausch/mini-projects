import socket
import select
import os
import yaml
import gzip
from jinja2 import Template
import traceback
import json
from log import Log
from html_builder import build_index, build_dir_page, build_log_page

class Server:
    def __init__(self):
        yaml_file="./config.yaml"
        if not os.path.isfile(yaml_file):
            print(f"No {yaml_file} file.\nPlease create a {yaml_file} file and fill it as per the README")
            exit(1)
        try:
            with open (yaml_file, "r") as file:
                data = yaml.safe_load(file)
                rawhostaddress = data["hostaddress"]
                rawhostport = data["hostport"]
                try:
                    hostport = int(rawhostport)
                except ValueError:
                    print(f"String \"{rawhostport}\" cannot be converted to INT")
                    exit()
                self.hostaddress = (rawhostaddress, hostport)
                raw_logs = data["logs"]
                self.logs = {}
                for raw_log in raw_logs:
                    log = Log(raw_log)
                    self.logs[log.name] = log
        except yaml.YAMLError as e:
            print(f"Error with YAML: {e}")
            exit(1)

    def create_json(self, path):
        log_index = path[2]
        new_path = "/".join(path[2:])
        log = self.logs[log_index]

        log_contents = {
            "contents": log.format_file_contents(log.log_file)
        }
        
        header = ("HTTP/1.1 200 OK\r\n"
            "Content-Type: text/html\r\n"
            "Set-Cookie: ServerName=logviewer\r\n"
            "\r\n")

        return header + json.dumps(log_contents)



    def create_html(self, path):
        log_index = path[1]
        new_path = "/".join(path[1:])
        back_path = "/".join(path[:-1]) if len(path) > 2 else "/"
        log = self.logs[log_index]
        log_dir = os.path.isdir(log.log_file)
        if log_dir:
            real_path = "/".join(path[3:])
            file_path = "/".join((log.log_file, real_path))

            if os.path.isfile(file_path):
                http_response = build_log_page(log, log_file=file_path, back_path=back_path)
            elif os.path.isdir(file_path):
                http_response = build_dir_page(log, file_path, new_path, back_path = back_path)
            else:
                print("-" * 50)
                print("ERROR")
                print(f"File Path: {file_path}\nReal Path: {real_path}")
                print("-" * 50)
        else:
            return build_log_page(log, back_path=back_path)
        try:
            if http_response is None:
                return
            else:
                return http_response
        except UnboundLocalError:
            return

    def run(self, stop_event=None):
        listener_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        listener_socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)

        listener_socket.bind(self.hostaddress)
        listener_socket.listen(1)
        print(f"Server listening @ {self.hostaddress[0]}:{self.hostaddress[1]}")

        index_html = build_index(self.logs)

        while not (stop_event and stop_event.is_set()):
            try:
                read_ready_sockets, _, _ = select.select([listener_socket], [], [], 1)  # Wait up to 1 second

                for ready_socket in read_ready_sockets:
                    client_socket, _client_address = ready_socket.accept()

                    message = client_socket.recv(4096)
                    try:
                        path = message.split()[1].decode('utf-8')
                    except IndexError as e:
                        print(f"INDEX ERROR:\n{e}")
                    if path == "" or path == "/":
                        http_response = index_html
                        print("INDEX")
                    elif "/log/" in path:
                        try:
                            path_sections = path.split("/")[1:]
                            if path_sections[0] == "api":
                                response = self.create_json(path_sections)
                            else:
                                print(path)
                                response = self.create_html(path_sections)
                            if response:
                                http_response = response
                            else: 
                                http_response = index_html
                        except IndexError:
                            print("Index out of range")
                            http_response = index_html

                    client_socket.sendall(http_response.encode())

                    try:
                        client_socket.close()
                    except OSError:
                        pass
            except KeyboardInterrupt:
                print("\n\nKeyboard Interrupt detected.\nExiting Gracefully...")
                exit()
            except Exception as e:
                print(traceback.print_exception(e))


if __name__ == "__main__":
    Server().run()
