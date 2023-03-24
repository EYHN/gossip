import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import ReactDOM from 'react-dom/client'
import { Graph } from './graph'
import { DataViewer } from './data-viewer'
import { Card, Container, Divider, Grid, IconButton, Paper, Slider, Stack } from '@mui/material'
import PauseIcon from '@mui/icons-material/Pause';
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

rust
  .then((m) => m.default)
  .then((m) => m.createSimulator())
  .then((simulator) => {
    const App = () => {
      const [state, setState] = useState<ViewState>()
      const [selectId, setSelectId] = useState<string | null>(null)
      const [speed, setSpeed] = useState<number>(10)
      const [isPause, setPause] = useState<boolean>(false)
      const viewerRef = useRef<VirtuosoHandle>(null)

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
      }, [speed, isPause])

      const handleUpdateData = useCallback((id: string, k: string, v: string) => {
        simulator.set_kv(id, k, v)
        const state = simulator.debug() as ViewState
        setState(state)
      }, [])

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
              <Grid xs={12} sm={12} md={4} lg sx={{ padding: '16px' }}>
                <h3>Gossip + CTDT Simulator</h3>
              </Grid>
              <Grid xs={12} sm={6} md={5} lg={4} sx={{ padding: '16px' }}>
                <Stack spacing={2} direction="column">
                  <Card variant="outlined" sx={{ maxWidth: '300px' }} >
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
                        <PauseIcon />
                      </IconButton>
                    </Stack>
                  </Card>
                  <Card sx={{ width: '300px' }}>
                    <Graph
                      state={{ ...state, selectId }}
                      onSelect={handleSelectClient}
                    />
                  </Card>
                </Stack>
              </Grid>
              <Grid xs={12} sm={6} md={3} sx={{ maxHeight: '100vh', padding: '16px 0' }}>
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
