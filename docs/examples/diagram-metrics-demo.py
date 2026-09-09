"""Verify Northwind diagram-load collection against an isolated local mock."""
import http.server,json,threading,subprocess,os,time,urllib.request
from pathlib import Path
root=Path(__file__).resolve().parents[2]; output=root/'.scryr/runtime-metrics-verification'; output.mkdir(parents=True,exist_ok=True)
calls=[]
class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        from urllib.parse import urlparse,parse_qs
        p=parse_qs(urlparse(self.path).query);calls.append(p)
        assert self.headers['Authorization']=='Basic ZGVtbzpkZW1v'
        expr=p['query'][0];value='120' if 'histogram_quantile' in expr else '2'
        body=json.dumps({'status':'success','data':{'resultType':'matrix','result':[{'values':[[int(p['end'][0]),value]]}]}}).encode()
        self.send_response(200);self.send_header('Content-Type','application/json');self.send_header('Content-Length',str(len(body)));self.end_headers();self.wfile.write(body)
    def log_message(self,*args): pass
backend=http.server.ThreadingHTTPServer(('127.0.0.1',0),Handler)
threading.Thread(target=backend.serve_forever,daemon=True).start()
connections=output/'mock-connections.json'
connections.write_text(json.dumps({'local-dev-org':{'northwind-grafana-read':{'endpoint':f'http://127.0.0.1:{backend.server_port}','username':'demo','token':'demo'}}}))
endpoint='http://127.0.0.1:8002/graphql'
env=dict(os.environ,SCRYR_SQLITE_PATH=str(output/'scryr.db'),SCRYR_METRICS_CONNECTIONS_FILE=str(connections))
cli=str(root/'crystal/target/debug/scryr'); log=(output/'server.log').open('w')
server=subprocess.Popen([cli,'serve','--port','8002','--auth-mode','local'],cwd=root,env=env,stdout=log,stderr=log)
def query(q):
    req=urllib.request.Request(endpoint,data=json.dumps({'query':q}).encode(),headers={'Content-Type':'application/json'})
    with urllib.request.urlopen(req,timeout=30) as r: value=json.load(r)
    assert not value.get('errors'),value
    return value['data']
try:
    for _ in range(100):
        try:query('{ __typename }');break
        except OSError:time.sleep(.1)
    project=root.parent/'ex-northwind-commerce'
    subprocess.run([cli,'generate','upload','--path',str(project/'index.scry'),'--manifest-dir',str(project),'--scryr-dir',str(root/'.scryr')],cwd=root,env=dict(env,SCRYR_GRAPHQL_URL=endpoint),check=True)
    blocks='{ blocks(scryIdentifier:"northwind_commerce_diagram") { name rawJsonString } }'
    for _ in range(3):query(blocks)
    assert len(calls)==0, 'ordinary block reads fetched metrics'
    metrics='{ diagramMetrics(scryIdentifier:"northwind_commerce_diagram") }'
    snapshot=query(metrics)['diagramMetrics']['northwind-commerce/api']
    assert snapshot['status']=='ready',snapshot
    assert len(calls)==5,len(calls)
    assert snapshot['values']['responseTimeP95']['value']==120
    assert snapshot['values']['responseTimeP95']['unit']=='ms'
    assert 'cpuCurrent' not in snapshot['values']
    query(metrics)
    for _ in range(3):query(blocks)
    assert len(calls)==5,'cache or polling contract broken'
    assert 'demo' not in json.dumps(snapshot)
    (output/'snapshot.json').write_text(json.dumps(snapshot,indent=2))
    print(json.dumps({'ordinaryBlockReadFetches':0,'diagramLoadQueries':5,'afterCachedLoadAndMorePolling':len(calls),'manifest':'northwind-commerce/api','status':snapshot['status'],'backend':'local mock, not live Grafana'},indent=2))
finally:
    server.terminate();server.wait(timeout=15);backend.shutdown();log.close()
