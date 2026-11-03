#!/usr/bin/env python3
"""Diagnostic-only RV32IM breakdown on the verified native recursion Metal worker."""
import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import subprocess
import sys
import uuid
from proof_provenance import ROOT, sha, verify_build
from worker_provenance import verify_build as verify_worker


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--engine',type=Path,required=True)
    parser.add_argument('--worker-kind',choices=['native-recursion-metal','native-rv32im-metal'],default='native-recursion-metal')
    args=parser.parse_args()
    worker=verify_worker(args.worker_kind,args.engine)
    verifier=verify_build('verifier')
    directory=ROOT/'target/macos/step6-assessment/profile-runs'/uuid.uuid4().hex
    directory.mkdir(parents=True)
    common=[sys.executable,str(ROOT/'scripts/macos/test-native-worker.py'),'--engine',str(args.engine),
        '--worker-kind',args.worker_kind,'--segment-hal','metal' if args.worker_kind=='native-rv32im-metal' else 'cpu','--recursion-hal','metal','--po2','20',
        '--repeat','1','--proof-only','--min-guest-cycles','10000000','--timeout-seconds','1800']
    cases=[('arithmetic-10m',['--workload','fibonacci','--iterations','1000000']),
           ('hash-10m',['--workload','sha256-chain','--iterations','2262'])]
    report={'status':'running','scope':'RISC0_VERBOSE=1 diagnostic timings; not a controlled speedup comparison or warm median',
        'started':datetime.now(timezone.utc).isoformat(),'worker_build':worker,'verifier_build':verifier,
        'script_sha256':sha(Path(__file__)),'runner_sha256':sha(ROOT/'scripts/macos/test-native-worker.py'),
        'cases':[dict(name=name,command=common+options,status='pending') for name,options in cases]}
    (directory/'plan.json').write_text(json.dumps(report,indent=2)+'\n')
    (directory/'driver-source.py').write_bytes(Path(__file__).read_bytes())
    print(directory,flush=True)
    try:
        for case in report['cases']:
            case.update(status='running',started=datetime.now(timezone.utc).isoformat())
            (directory/'result.json').write_text(json.dumps(report,indent=2)+'\n')
            with (directory/(case['name']+'.log')).open('w') as out:
                subprocess.run(case['command'],cwd=ROOT,env=dict(os.environ,RISC0_VERBOSE='1'),
                    stdout=out,stderr=subprocess.STDOUT,check=True)
            run=Path((directory/(case['name']+'.log')).read_text().splitlines()[0])
            result=json.loads((run/'result.json').read_text());assert result['status']=='passed'
            assert result['worker_build']==worker and result['verifier_build']==verifier
            assert result['results']['verified_proofs']==1
            case.update(status='passed',run=str(run),result_sha256=sha(run/'result.json'))
            print(case['name'],'passed',flush=True)
        assert verify_worker(args.worker_kind,args.engine)==worker
        assert verify_build('verifier')==verifier
        report['status']='completed'
    except BaseException as error:
        report.update(status='failed',error=str(error))
        for case in report['cases']:
            if case['status']=='running':case['status']='failed'
        raise
    finally:
        report['finished']=datetime.now(timezone.utc).isoformat()
        (directory/'result.json').write_text(json.dumps(report,indent=2)+'\n')


if __name__=='__main__':main()
