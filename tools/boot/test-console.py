#!/usr/bin/env python3
"""Interact through COM1, verify preemption, isolation, IPC, teardown; no host disks/NIC."""
import argparse,hashlib,json,re,select,shutil,socket,subprocess,tempfile,time
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--out',type=Path,required=True);p.add_argument('--uefi',action='store_true');p.add_argument('--stress-seconds',type=int,default=0);a=p.parse_args();out=a.out.resolve()
with tempfile.TemporaryDirectory(prefix='haios-console-')as directory:
    work=Path(directory);serial=work/'serial.sock';qmp=work/'qmp.sock'
    command=['qemu-system-x86_64','-machine','q35,accel=kvm','-cpu','host','-smp','1','-m','256M','-display','none','-monitor','none','-serial','unix:'+str(serial)+',server=on,wait=on','-qmp','unix:'+str(qmp)+',server=on,wait=off','-nic','none','-no-reboot','-no-shutdown','-boot','d','-drive','file='+str(out/'haios.iso')+',media=cdrom,readonly=on,format=raw']
    if a.uefi:
        shutil.copyfile('/usr/share/OVMF/OVMF_VARS_4M.fd',work/'vars.fd');command+=['-drive','if=pflash,format=raw,unit=0,readonly=on,file=/usr/share/OVMF/OVMF_CODE_4M.fd','-drive','if=pflash,format=raw,unit=1,file='+str(work/'vars.fd')]
    transcript=bytearray();checks=[];events=[];channel=None;monitor=None
    with (work/'stderr').open('wb')as err:
        process=subprocess.Popen(command,stderr=err,stdout=subprocess.DEVNULL)
        try:
            deadline=time.monotonic()+10
            while not serial.exists():
                if process.poll()is not None or time.monotonic()>deadline:raise RuntimeError('No serial socket')
                time.sleep(.02)
            channel=socket.socket(socket.AF_UNIX);channel.connect(str(serial));channel.setblocking(False)
            def wait(pattern,start=0,timeout=15):
                deadline=time.monotonic()+timeout
                while time.monotonic()<deadline:
                    found=re.search(pattern,bytes(transcript[start:]))
                    if found:return found
                    if process.poll()is not None:raise RuntimeError('Guest exited')
                    if select.select([channel],[],[],.05)[0]:
                        data=channel.recv(65536)
                        if not data:raise RuntimeError('Serial closed')
                        transcript.extend(data)
                raise RuntimeError('Console timeout: '+repr(pattern))
            def send(text,pattern):
                start=len(transcript);channel.sendall(text.encode()+b'\n');return wait(pattern,start)
            wait(rb'haios> ',timeout=25)
            assert b'HAIOS:CONSOLE:READY' in transcript and b'HAIOS:PMM:OK' in transcript
            checks.append('boot/PMM/interactive prompt')
            send('help',rb'help calc')
            send('version',rb'HAIOS 0.3.0-dev console')
            base=int(send('mem',rb'MEM free_frames=(\d+) page_bytes=').group(1))
            start=len(transcript);channel.sendall(b'run hello\n');wait(rb'hello:world\n',start);wait(rb'PROC:EXIT pid=\d+',start)
            assert int(send('mem',rb'MEM free_frames=(\d+) page_bytes=').group(1))==base
            checks.append('hello and complete frame reclamation')
            start=len(transcript);channel.sendall(b'run count\nrun count\n');wait(rb'count:done\n[\s\S]*count:done\n',start)
            wait(rb'PROC:EXIT pid=\d+\n[\s\S]*PROC:EXIT pid=\d+',start)
            assert int(send('mem',rb'MEM free_frames=(\d+) page_bytes=').group(1))==base
            checks.append('two concurrent processes and reclamation')
            spin=int(send('run spin',rb'PROC:START pid=(\d+) name=spin').group(1));time.sleep(.1)
            switches1=int(send('ps',str(spin).encode()+rb' spin (\d+)\n').group(1));time.sleep(.1)
            switches2=int(send('ps',str(spin).encode()+rb' spin (\d+)\n').group(1));assert switches2>switches1>0
            checks.append('preemption of uncooperative ring3 spin')
            start=len(transcript);channel.sendall(b'run fault\n');wait(rb'EXCEPTION vector=14 error=5 pid=',start);wait(rb'PROC:EXIT pid=\d+',start)
            send('ps',str(spin).encode()+rb' spin \d+')
            send('kill '+str(spin),rb'PROC:EXIT pid='+str(spin).encode())
            assert int(send('mem',rb'MEM free_frames=(\d+) page_bytes=').group(1))==base
            checks.append('kernel memory protection; unaffected process/console')
            start=len(transcript);channel.sendall(b'run writefault\n');wait(rb'EXCEPTION vector=14 error=7 pid=',start);wait(rb'PROC:EXIT pid=\d+',start)
            start=len(transcript);channel.sendall(b'run badptr\n');wait(rb'badptr:ok\n',start);wait(rb'PROC:EXIT pid=\d+',start)
            checks.append('RX protection and syscall kernel/overflow/cross-page/length rejection')
            start=len(transcript);channel.sendall(b'run fpu\n');wait(rb'EXCEPTION vector=7 error=0 pid=',start);wait(rb'PROC:EXIT pid=\d+',start)
            checks.append('unsupported FPU trapped instead of sharing register state')
            start=len(transcript);channel.sendall(b'run ipc\n');wait(rb'ipc:sent\n',start);wait(rb'ipc:received\n',start)
            wait(rb'PROC:EXIT pid=\d+\n[\s\S]*PROC:EXIT pid=\d+',start)
            assert int(send('mem',rb'MEM free_frames=(\d+) page_bytes=').group(1))==base
            checks.append('IPC FIFO demonstration and teardown')
            send('wrong',rb'ERROR command');send('x'*97,rb'ERROR line too long');send('ls',rb'RAM: hello');send('cat about',rb'HAIOS 0.3:')
            checks.append('invalid/long commands and RAM catalog')
            # Four fixed slots; failure creating an IPC pair must unwind its first child.
            spins=[int(send('run spin',rb'PROC:START pid=(\d+) name=spin').group(1))for _ in range(3)]
            limited=int(send('mem',rb'MEM free_frames=(\d+) page_bytes=').group(1));send('run ipc',rb'ERROR process capacity or memory')
            assert int(send('mem',rb'MEM free_frames=(\d+) page_bytes=').group(1))==limited
            spins.append(int(send('run spin',rb'PROC:START pid=(\d+) name=spin').group(1)))
            send('run hello',rb'ERROR unknown program, capacity or memory')
            for pid in spins:send('kill '+str(pid),rb'PROC:EXIT pid='+str(pid).encode())
            assert int(send('mem',rb'MEM free_frames=(\d+) page_bytes=').group(1))==base
            checks.append('process capacity, partial-create rollback, final resource balance')
            stress=None
            if a.stress_seconds:
                if a.stress_seconds<3600:raise ValueError('Acceptance stress requires at least 3600 seconds')
                spin=int(send('run spin',rb'PROC:START pid=(\d+) name=spin').group(1))
                steady=int(send('mem',rb'MEM free_frames=(\d+) page_bytes=').group(1))
                assert steady==base-7
                beginning=time.monotonic();cycles=0;samples=[];next_report=0.0
                while time.monotonic()-beginning<a.stress_seconds:
                    start=len(transcript);channel.sendall(b'run count\nrun count\n')
                    wait(rb'count:done\n[\s\S]*count:done\n',start)
                    wait(rb'PROC:EXIT pid=\d+\n[\s\S]*PROC:EXIT pid=\d+',start)
                    start=len(transcript);channel.sendall(b'run ipc\n')
                    wait(rb'ipc:sent\n',start);wait(rb'ipc:received\n',start)
                    wait(rb'PROC:EXIT pid=\d+\n[\s\S]*PROC:EXIT pid=\d+',start)
                    start=len(transcript);channel.sendall(b'run fault\n')
                    wait(rb'EXCEPTION vector=14 error=5 pid=',start);wait(rb'PROC:EXIT pid=\d+\n',start)
                    free=int(send('mem',rb'MEM free_frames=(\d+) page_bytes=').group(1))
                    assert free==steady,(cycles,free,steady)
                    cycles+=1;elapsed=time.monotonic()-beginning
                    if elapsed>=next_report:
                        sample={'elapsed_seconds':round(elapsed,2),'cycles':cycles,'free_frames':free}
                        samples.append(sample);print(json.dumps({'stress_progress':sample}),flush=True)
                        next_report=elapsed+30
                send('kill '+str(spin),rb'PROC:EXIT pid='+str(spin).encode()+rb'\n')
                final=int(send('mem',rb'MEM free_frames=(\d+) page_bytes=').group(1));assert final==base
                stress={'duration_seconds':time.monotonic()-beginning,'cycles':cycles,'background_spin':True,'steady_free_frames':steady,'final_free_frames':final,'samples':samples,'workload':'two count processes, IPC pair (MAX u64 and 42), deliberate user page fault, full frame balance each cycle'}
                checks.append('one continuous hour of processes/IPC/faults with invariant resource balance')
            assert b'KERNEL:FAULT'not in transcript and b'HAIOS:PANIC'not in transcript
            monitor=socket.socket(socket.AF_UNIX);monitor.settimeout(2);monitor.connect(str(qmp));stream=monitor.makefile('rwb',buffering=0);json.loads(stream.readline())
            def request(name):
                stream.write((json.dumps({'execute':name})+'\n').encode())
                while True:
                    response=json.loads(stream.readline())
                    if 'event'in response:events.append(response);continue
                    assert 'error'not in response,response
                    return response.get('return')
            request('qmp_capabilities');status=request('query-status');assert status['status']=='running'
            assert not any(e['event']in ('RESET','SHUTDOWN','GUEST_PANICKED')for e in events)
            request('quit');process.wait(timeout=5);assert process.returncode==0
            report={'passed':True,'firmware':'UEFI'if a.uefi else'BIOS','checks':checks,'baseline_free_frames':base,'iso_sha256':hashlib.sha256((out/'haios.iso').read_bytes()).hexdigest(),'serial':transcript[-65536:].decode(errors='replace'),'serial_sha256':hashlib.sha256(transcript).hexdigest(),'stress':stress,'qemu_command':command,'qmp_status':status,'termination_events':events,'exit':process.returncode}
            if a.stress_seconds:
                import gzip
                (out/'stress-transcript.txt.gz').write_bytes(gzip.compress(bytes(transcript)))
            (out/('test-console-'+report['firmware'].lower()+('-stress' if a.stress_seconds else '')+'.json')).write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items()if k!='serial'},indent=2))
        except Exception:
            (out/'failed-console.log').write_bytes(transcript);print(transcript.decode(errors='replace'));print((work/'stderr').read_text());raise
        finally:
            if channel:channel.close()
            if monitor:monitor.close()
            if process.poll()is None:process.kill();process.wait(timeout=5)
