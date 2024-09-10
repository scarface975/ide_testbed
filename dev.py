#!/usr/bin/env python3

import shutil
import subprocess
import os
import re
import selenium.webdriver
import signal
import socketserver
import time
import threading
import watchdog.observers
import watchdog.events
import http
import http.server
import errno

from typing import Tuple

project_path = os.getcwd()
build_path = project_path + '/build'
crates_path = project_path + '/crates'
static_path = project_path + '/static'
default_port = 3000

def build():
    ### Set up the build directory
    if not os.path.isdir(build_path):
        os.mkdir(build_path)
    if not os.path.islink(build_path + '/package.json'):
        os.symlink('../package.json', build_path + '/package.json')
    if not os.path.islink(build_path + '/package-lock.json'):
        os.symlink('../package-lock.json', build_path + '/package-lock.json')
    if not os.path.islink(build_path + '/rspack.config.js'):
        os.symlink('../rspack.config.js', build_path + '/rspack.config.js')
    ### Run npm install
    print('Fetching node packages')
    npm_args = ['npm', 'install']
    npm_proc = subprocess.Popen(npm_args,
        cwd=build_path,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL)
    ### Run cargo build
    print('Building with Cargo')
    cargo_args = [
        'cargo',
        'build'
    ]
    for package in ['frontend']:
        cargo_args.extend(['--package', package])
    cargo_proc = subprocess.Popen(cargo_args, cwd='crates')
    npm_proc.wait()
    cargo_proc.wait()
    if cargo_proc.returncode != 0:
        raise RuntimeError('Cargo terminated with {}'.format(cargo_proc.returncode))
    if npm_proc.returncode != 0:
        raise RuntimeError('npm terminated with {}'.format(npm_proc.returncode))
    ### Run wasm-bindgen
    print('Generating bindings')
    artifact_path = '{}/artifacts/debug'.format(build_path)
    wasm_bindgen_procs = []
    for package in ['frontend']:
        wasm_bindgen_args = [
            'wasm-bindgen',
            '--reference-types',
            '--no-typescript',
            '--out-dir', artifact_path
        ]
        wasm_bindgen_args.append(build_path + '/crates/wasm32-unknown-unknown/debug/' + package + '.wasm')
        wasm_bindgen_procs.append(subprocess.Popen(wasm_bindgen_args))
    for wasm_bindgen_proc in wasm_bindgen_procs:
        wasm_bindgen_proc.wait()
        if wasm_bindgen_proc.returncode != 0:
            raise RuntimeError('wasm-bindgen terminated with {}'.format(wasm_bindgen_proc.returncode))
    ### Prepare output 
    dist_path = '{}/dist/debug'.format(build_path)
    shutil.rmtree(dist_path, ignore_errors=True)
    os.makedirs(dist_path)
    ### Run Rspack
    print('Bundling frontend')
    rspack_args = [
        'npx', 'rspack', 'build',
        '--entry', '{}/{}.js'.format(artifact_path, 'frontend'),
        '--output-path', dist_path,
        '--mode', 'development'
    ]
    rspack_proc = subprocess.Popen(rspack_args, cwd=build_path)
    rspack_proc.wait()
    if rspack_proc.returncode != 0:
        raise RuntimeError('rspack terminated with {}'.format(rspack_proc.returncode))
    ### Run Node-Sass
    print('Generating styles')
    node_sass_args = [
        'npx',
        'node-sass',
        '--omit-source-map-url',
        project_path + '/styles.scss',
        dist_path + '/styles.css'
    ]
    node_sass_proc = subprocess.Popen(node_sass_args, cwd=build_path)
    node_sass_proc.wait()
    if node_sass_proc.returncode != 0:
        raise RuntimeError('node-sass terminated with {}'.format(node_sass_proc.returncode))
    ### Copy static assets
    print('Copying static assets')
    shutil.copytree(static_path, dist_path, dirs_exist_ok=True)
    return dist_path

def watch():
    class EventHandler(watchdog.events.FileSystemEventHandler):
        def __init__(self, observer, *args, **kwargs):
            self.observer = observer
            super().__init__(*args, **kwargs)
        def on_modified(self, event):
            self.observer.stop()
    
    observer = watchdog.observers.Observer()
    observer.schedule(EventHandler(observer), crates_path, recursive=True)
    observer.schedule(EventHandler(observer), static_path, recursive=True)
    observer.start()
    try:
        observer.join()
    except KeyboardInterrupt:
        observer.stop()
        observer.join()
        raise KeyboardInterrupt

def serve() -> Tuple[socketserver.TCPServer, int]:
    class RequestHandler(http.server.SimpleHTTPRequestHandler):
        def __init__(self, *args, **kwargs):
            directory = os.path.join(project_path, 'build', 'dist', 'debug')
            super().__init__(*args, directory=directory, **kwargs)
        def log_message(self, format, *args):
            pass
        def end_headers(self):
            self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
            self.send_header("Cross-Origin-Opener-Policy", "same-origin")
            http.server.SimpleHTTPRequestHandler.end_headers(self)
    ### Find an available port to serve on
    current_port = 3000
    while current_port < 3016:
        try:
            server = socketserver.TCPServer(('localhost', current_port), RequestHandler)
        except OSError:
            current_port += 1
        else:
            thread = threading.Thread(target=server.serve_forever)
            thread.start()
            return server, current_port
    raise OSError(errno.EADDRINUSE, os.strerror(errno.EADDRINUSE))


server = None
driver = None
try:
    options = selenium.webdriver.ChromeOptions()
    driver = selenium.webdriver.Remote(
        command_executor='http://127.0.0.1:4444/wd/hub',
        options=options,
        desired_capabilities=dict())
except Exception as e:
    print('Could not connect to Selenium Grid') 

try:
    while True:
        try:
            if server:
                server.shutdown()
                server = None
            build()
            server, port = serve()
            time.sleep(0.5)
            if driver:
                try:
                    driver.get('http://localhost:{}/index.html'.format(port))
                except Exception as error:
                    try:
                        options = selenium.webdriver.ChromeOptions()
                        driver = selenium.webdriver.Remote(
                            command_executor='http://127.0.0.1:4444/wd/hub',
                            options=options,
                            desired_capabilities=dict())
                        driver.get('http://localhost:{}/index.html'.format(port))
                    except Exception as e:
                        driver = None
                        print('Could not reload URL')
            else:
                pass
        except RuntimeError:
            pass
        watch()
except KeyboardInterrupt:
    pass
if driver:
    try:
        driver.close()
    except Exception:
        pass
if server:
    server.shutdown()
