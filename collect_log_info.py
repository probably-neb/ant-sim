#!/bin/env python3
import types
import matplotlib
import sys
import time
import os
import re
from datetime import datetime
import subprocess
from dataclasses import dataclass

TIME_SECS = 600

RUN_NAME = ""
RUN_DIR = ""

if len(sys.argv) < 2:
    print("run name required")
    sys.exit(1)
else: 
    RUN_NAME = sys.argv[1]
    RUN_NAME = RUN_NAME.strip(".")
    RUN_NAME = RUN_NAME.strip("/")
    RUN_NAME = RUN_NAME.removeprefix("data/")
    RUN_DIR = "./data/" + RUN_NAME
    try:
        os.mkdir(RUN_DIR)
    except FileExistsError:
        pass

def shell(cmd, **kwargs):
    return not subprocess.run(cmd, shell=True, **kwargs)

# https://www.programcreek.com/python/?CodeExample=strip+ansi
def strip_ansi(source):
    """
    Remove ansi escape codes from text.
    
    Parameters
    ----------
    source : str
        Source to remove the ansi from
    """
    return re.sub(r'\033\[(\d|;)+?m', '', source)

def parse_time(time_str):
    dt = datetime.strptime(strip_ansi(time_str.strip()), "%Y-%m-%dT%H:%M:%S.%fZ")
    return dt

@dataclass
class LineParser:
    getter: types.FunctionType
    outfile_name: str
    outfile: None = None
    header: str = ""

    def __post_init__(self):
        self.outfile_name = os.path.join(RUN_DIR, self.outfile_name)
        self.outfile = open(self.outfile_name, 'w')
        if self.header:
            self.outfile.write(self.header+'\n')

    def parse(self, line, *args, **kwargs):
        csv = self.getter(line, obj=self, *args, **kwargs)
        if csv is not None:
            self.outfile.write(csv+'\n')

PARSERS = []

def parse_fps(line, obj=None, *args, **kwargs):
    csv = None
    if 'fps' in line:
        tokens = line.split()
        time_str, fps, avg = tokens[0], tokens[7], tokens[9][:-1]
        time_offset = (parse_time(time_str) - obj.first_time)
        # time_offset = parse_time(time_str)
        secs = time_offset.total_seconds()
        # secs = time_offset.timestamp()
        csv = f'{secs},{fps},{avg}'
    return csv

fps_parser = LineParser(outfile_name="fps.csv", getter=parse_fps, header = "time,fps,avg_fps")
PARSERS.append(fps_parser)

def parse_entity_count(line, obj=None, *args, **kwargs):
    csv = None
    if 'entity_count' in line:
        tokens = line.split()
        time_str, num_entities, avg_entities = tokens[0], tokens[7], tokens[9][:-1]
        time_offset = (parse_time(time_str) - obj.first_time)
        secs = time_offset.total_seconds()
        csv = f'{secs},{num_entities},{avg_entities}'
    return csv
entity_parser = LineParser(outfile_name="entity_count.csv", getter=parse_entity_count, header="time,num_entities,avg_entities")
PARSERS.append(entity_parser)

def parse_reached_dest(line, obj=None, *args, **kwargs):
    csv = None
    if 'Ant reached' in line:
        # print('ANT REACHED:',line)
        tokens = line.split()
        time_str, dest_type, steps = tokens[0], tokens[6], int(tokens[10])
        obj.totals[dest_type]['sum']+=steps
        obj.totals[dest_type]['num']+=1
        sum_ = obj.totals[dest_type]['sum']
        num_ = obj.totals[dest_type]['num']
        avg_ = sum_/num_
        obj.totals[dest_type]['avg']=avg_
        time_offset = (parse_time(time_str) - obj.first_time)
        secs = time_offset.total_seconds()
        csv = f'{secs},{dest_type},{steps},{avg_}'
    return csv
reached_dest_parser = LineParser(outfile_name="reached_dest.csv", getter=parse_reached_dest, header="time,dest_type,steps,avg_steps")
reached_dest_parser.totals = {'parent': {'sum': 0, 'num': 0, 'avg': 0}, 'target': {'sum': 0, 'num': 0, 'avg': 0}}
PARSERS.append(reached_dest_parser)
# logic

shell("cargo build --profile=dev", check=True)

    # shell(f"cargo --color=never flamegraph -o {RUN_DIR}/flamegraph.svg > output.txt", timeout=10)
TOKENIZE=False
import shlex
# try:
cmd = shlex.split(f"cargo --color never flamegraph --dev -o {RUN_DIR}/flamegraph-{RUN_NAME}.svg")
with open('output.txt', 'w') as out:
    proc = subprocess.Popen(cmd, start_new_session=True, stdout=out, text=True)
    # proc.wait(TIME_SECS)
    time.sleep(TIME_SECS)
    # raise subprocess.TimeoutExpired(None,None)
# except subprocess.TimeoutExpired:
# proc.terminate()
    import signal
    proc_gpid = os.getpgid(proc.pid)
    os.kill(proc.pid, signal.SIGINT)
    # wait for flamegraph to finish
    time.sleep(10)
    os.killpg(proc_gpid, signal.SIGTERM)
# proc.send_signal(os.)
    with open('output.txt', 'r') as out:
        first_time = None
        if TOKENIZE:
            for line in out.readlines():
                print(list(enumerate(line.split())))
        else:
            for line in out.readlines():
                if not first_time and 'INFO' in line or 'WARN' in line:
                    first_time_str = line.split()[0]
                    first_time = parse_time(first_time_str)
                    for parser in PARSERS:
                        parser.first_time = first_time
                for parser in PARSERS:
                    parser.parse(line)

# with open(sys.argv[1], 'w') as out:
#     first_time = None
#     header = "time,fps,avg_fps"
#     while True:
#         line = input()
#         if not first_time and 'INFO' in line:
#             first_time_str = line.split()[0]
#             first_time = parse_time(first_time_str)
#         if 'fps' in line:
#             tokens = line.split()
#             time_str, fps, avg = tokens[0], tokens[7], tokens[9]
#             time_offset = (parse_time(time_str) - first_time)
#             secs = time_offset.total_seconds()
#             csv = f'{secs},{fps},{avg[:-1]}'
#             print(csv)
#             out.write(csv + '\n')
#         # print("SEP")
