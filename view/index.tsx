import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import ReactDOM from 'react-dom/client'
import { Graph } from './graph'
import { DataViewer } from './data-viewer'
import { Button, Card, Container, Divider, FormControl, Grid, IconButton, InputLabel, MenuItem, Paper, Select, SelectChangeEvent, Slider, Stack, TextField } from '@mui/material'
import PauseIcon from '@mui/icons-material/Pause';
import PlayIcon from '@mui/icons-material/PlayArrow';
import SpeedIcon from '@mui/icons-material/Speed';
import { css, Global } from '@emotion/react'
import { VirtuosoHandle } from 'react-virtuoso'

const rust = import('../crates/view/pkg')

export interface ViewState {
  clients: {
    id: string
    hash: string
    progress: number
  }[]
  messages: {
    from: string
    to: string
    progress: number
    kind: string
  }[]
  time: number
}

function parseNonzeroInt(v: string) {
  const n = parseInt(v)
  if (isNaN(n) || n <= 0) {
    return 1
  }
  return n
}

function parseUnsignedFloat(v: string) {
  const n = parseFloat(v)
  if (isNaN(n) || n < 0) {
    return 0
  }
  return n
}

rust
  .then((m) => m.default)
  .then((m) => {
    const App = () => {
      const [state, setState] = useState<ViewState>()
      const [selectId, setSelectId] = useState<string | null>(null)
      const [speed, setSpeed] = useState<number>(10)
      const [isPause, setPause] = useState<boolean>(false)
      const [options, setOptions] = useState({
        num_nodes: 3,
        fanout: 1,
        message_delay: 1,
        client_timer: 5,
        client_timer_random: 3,
        protocol_mode: 'pull'
      })
      const viewerRef = useRef<VirtuosoHandle>(null)
      const [runningOptions, setRunningOptions] = useState<typeof options>(options)

      const simulator = useMemo(() => {
        return m.create_simulator(runningOptions)
      }, [runningOptions])

      const createSetter = useCallback((key: keyof typeof options, converter?: (v: string) => any) => {
        return (e: { target: { value: string } }) => {
          setOptions((prev) => ({
            ...prev,
            [key]: converter ? converter(e.target.value) : e.target.value
          }))
        }
      }, [])

      useEffect(() => {
        if (isPause) {
          return
        }
        let animationFrameRequest = 0
        let prevTime: number | null = null
        const loop = (time: DOMHighResTimeStamp) => {
          if (prevTime === null) {
            prevTime = time
          } else {
            simulator.tick(((time - prevTime) / 1000) * (speed / 10))
            const state = simulator.debug() as ViewState
            setState(state)
            prevTime = time
          }
          animationFrameRequest = requestAnimationFrame(loop)
        }
        animationFrameRequest = requestAnimationFrame(loop)
        return () => cancelAnimationFrame(animationFrameRequest)
      }, [simulator, speed, isPause])

      const handleUpdateData = useCallback((id: string, k: string, v: string) => {
        simulator.set_kv(id, k, v)
        const state = simulator.debug() as ViewState
        setState(state)
      }, [simulator])

      const handleSelectClient = useCallback((selectId: string, index: number) => {
        setSelectId(selectId)
        viewerRef.current?.scrollToIndex({ index, behavior: 'smooth' })
      }, [])

      const handleSpeedChange = useCallback(
        (_: any, value: number | number[]) => {
          if (typeof value === 'number')
            setSpeed(
              Math.max(1, Math.min(100, value))
            )
        },
        []
      )

      const handleClickPause = useCallback(() => {
        setPause(!isPause)
      }, [isPause])

      const handleRestart = useCallback(() => {
        setRunningOptions({ ...options });
        setState(undefined);
        setLastConvergedTime(0);
        setConvergedTime(0);
      }, [options])

      const isConverged = useMemo(() => {
        if (!state) {
          return true
        }
        return !state.clients.some((c) => c.hash != state.clients[0].hash)
      }, [state])
      const [lastConvergedTime, setLastConvergedTime] = useState<number>(0)
      const [convergedTime, setConvergedTime] = useState<number>(0)

      useEffect(() => {
        if (state) {
          if (isConverged) {
            setLastConvergedTime(state.time)
          } else {
            setConvergedTime(state.time - lastConvergedTime)
          }
        }
      }, [isConverged, state])

      if (state) {
        return (
          <Container maxWidth={false} disableGutters>
            <Grid container spacing={2} sx={{ width: '100%', margin: 0, minHeight: '100vh', overflowY: 'auto' }} >
              <Global styles={css`
              body {
                margin: 0;
                background: #e7ebf0;
              }
              *,*:after,*:before {
                box-sizing: border-box;
              }
            `} />
              <Grid xs={12} sm={12} md={4} sx={{ padding: '16px', lineHeight: 1.6 }}>
                <h3>Gossip + CRDT Simulator</h3>
                <h4>Gossip Protocal</h4>
                <p>
                  The <a href="https://en.wikipedia.org/wiki/Gossip_protocol" target="_blank">gossip protocol</a> is a
                  decentralized method of information sharing in a network of clients. In the gossip protocol, each node
                  randomly selects a set of other clients to exchange information with. The clients exchange information and
                  update their own state.  The protocol is designed to efficiently disseminate information throughout
                  a network, even if individual clients are periodically unavailable.
                </p>
                <h4>
                  CRDT (Conflict-free Replicated Data Type)
                </h4>
                <p>
                  <a href="https://en.wikipedia.org/wiki/Conflict-free_replicated_data_type" target="_blank">CRDT</a> is a data type that
                  can be replicated across multiple clients and can be merged without conflict.
                </p>
                <h4>This Simulator</h4>
                <p>
                  This simulator is a visualization of the gossip protocol and CRDT. The simulator aimed to show every clients' state and
                  how the clients communicate with each other. And some parameters are provided to find a balance between convergence speed
                  and communication frequency.
                </p>
                <p>
                  Notice: There is not a standard gossip protocol implementation, and this implementation aims to achieve synchronized
                  states in unreliable communication and high leave rate clients.
                </p>
                <h4>Parameters</h4>
                <p>
                  <b>Node Count</b>: The number of clients in the network.<br />
                  <b>Fanout</b>: The number of clients that each client selects to exchange information with.<br />
                  <b>Message Delay</b>: The delay of message transmission.<br />
                  <b>Client Timer</b>: The interval of client to exchange information with other clients.<br />
                  <b>Client Timer Random ±</b>: The random interval of client to exchange information with other clients.<br />
                  <b>Mode</b>: In the gossip protocol, <i>push</i> and <i>pull</i> are two different ways that nodes can exchange information
                  with each other. <i>Push</i> means that a node proactively sends updated information to another node, while <i>pull</i> means
                  that a node requests updated information from another node, and <i>push-pull</i> means that a node can do both at the same
                  time. This simulator supports <i>PullOnly</i> and <i>PushPull</i> mode.<br />
                  <b>ADD DATA +</b>: add random data to a client.
                </p>
              </Grid>
              <Grid xs={12} sm={6} md={4} sx={{ padding: '16px' }}>
                <Stack spacing={2} direction="column">
                  <Card variant="outlined" sx={{ maxWidth: '300px' }} >
                    <Stack
                      spacing={2}
                      direction="row"
                      sx={{ m: 1 }}
                      alignItems="center"
                    >
                      <TextField
                        label="Node Count"
                        type="number"
                        size="small"
                        value={options.num_nodes}
                        onChange={createSetter('num_nodes', parseNonzeroInt)}
                        InputLabelProps={{
                          shrink: true,
                        }}
                      />
                      <TextField
                        label="Fanout"
                        type="number"
                        size="small"
                        value={options.fanout}
                        onChange={createSetter('fanout', parseNonzeroInt)}
                        InputLabelProps={{
                          shrink: true,
                        }}
                      />
                    </Stack>
                    <Stack
                      spacing={2}
                      direction="row"
                      sx={{ m: 1 }}
                      alignItems="center"
                    >
                      <TextField
                        label="Message Delay"
                        type="number"
                        size="small"
                        value={options.message_delay}
                        onChange={createSetter('message_delay', parseUnsignedFloat)}
                        InputLabelProps={{
                          shrink: true,
                        }}
                      />
                      <TextField
                        label="Client Timer"
                        type="number"
                        size="small"
                        value={options.client_timer}
                        onChange={createSetter('client_timer', parseUnsignedFloat)}
                        InputLabelProps={{
                          shrink: true,
                        }}
                      />
                    </Stack>
                    <Stack
                      spacing={2}
                      direction="row"
                      sx={{ m: 1 }}
                      alignItems="center"
                    >
                      <TextField
                        label="Client Timer Random ±"
                        type="number"
                        size="small"
                        value={options.client_timer_random}
                        onChange={createSetter('client_timer_random', parseUnsignedFloat)}
                        InputLabelProps={{
                          shrink: true,
                        }}
                        sx={{ flex: '1 1 50%' }}
                      />
                      <FormControl sx={{ flex: '1 1 50%' }}>
                        <InputLabel id="mode-select-label">Mode</InputLabel>
                        <Select
                          labelId="mode-select-label"
                          label="Mode"
                          size="small"
                          value={options.protocol_mode}
                          onChange={createSetter('protocol_mode')}
                        >
                          <MenuItem value="pull">PullOnly</MenuItem>
                          <MenuItem value="pushpull">PushPull</MenuItem>
                        </Select>
                      </FormControl>
                    </Stack>
                    <Stack
                      spacing={2}
                      direction="row"
                      sx={{ m: 1 }}
                      alignItems="center"
                    >
                      <Button size="small" variant="outlined" sx={{ flex: '1' }} onClick={handleRestart}>
                        Restart
                      </Button>
                    </Stack>
                    <Stack
                      divider={<Divider orientation="vertical" flexItem />}
                      spacing={2}
                      direction="row"
                      sx={{ m: 1 }}
                      alignItems="center"
                    >
                      <Stack spacing={2} sx={{ flexGrow: 1 }} direction="row" alignItems="center">
                        <SpeedIcon />
                        <Slider
                          aria-label="Speed"
                          value={speed}
                          min={1}
                          max={100}
                          onChange={handleSpeedChange}
                        />
                      </Stack>
                      <IconButton size='small' onClick={handleClickPause} aria-label="Pause">
                        {!isPause ? <PauseIcon /> : <PlayIcon />}
                      </IconButton>
                    </Stack>
                  </Card>
                  <Card sx={{ width: '300px' }}>
                    <Graph
                      state={{ ...state, selectId }}
                      onSelect={handleSelectClient}
                    />
                  </Card>
                  <Card sx={{ width: '300px' }}>
                    <Stack spacing={2} direction="column" sx={{ m: 1 }}>
                      Convergence Status: {isConverged ? 'Converged' : 'Not Converged'}<br />
                      Convergence Time: {(convergedTime).toFixed(2)}
                    </Stack>
                  </Card>
                </Stack>
              </Grid>
              <Grid xs={12} sm={6} md={4} sx={{ maxHeight: '100vh', padding: '16px 0' }}>
                <DataViewer
                  ref={viewerRef}
                  simulator={simulator}
                  state={state}
                  style={{ height: '100%', minHeight: '50vh' }}
                  onUpdate={handleUpdateData}
                />
              </Grid>
            </Grid>
          </Container >
        )
      }
      return <></>
    }

    ReactDOM.createRoot(document.body).render(<App />)
  })
  .catch(console.error)
