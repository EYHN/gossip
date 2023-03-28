import { faker } from '@faker-js/faker'
import AddIcon from '@mui/icons-material/Add';
import { Avatar, Box, Button, Card, Chip, IconButton, Stack } from '@mui/material'
import { JsonViewer } from '@textea/json-viewer'
import React from 'react'
import { Virtuoso, VirtuosoHandle } from 'react-virtuoso'
import { ViewState } from '.'
import type { ExportedSimulator } from '../crates/view/pkg'
import { selectColor } from './graph'

export const DataViewer = React.forwardRef<VirtuosoHandle, {
  simulator: ExportedSimulator
  state: ViewState
  style?: React.CSSProperties
  onUpdate?: (id: string, k: string, v: string) => void
}>(
  ({
    simulator,
    state,
    style,
    onUpdate,
  }, ref) => {
    return (
      <Virtuoso
        ref={ref}
        style={style}
        totalCount={state.clients.length}
        itemContent={(index) => {
          const client = state.clients[index]
          const messageCount = state.messages.filter(m => m.from === client.id || m.to === client.id).length
          return (
            <Box sx={{ height: 200, p: 1 }}>
              <Card sx={{ height: '100%', p: 1 }}>
                <Stack direction="column" sx={{ height: '100%' }}>
                  <Stack direction="row" alignItems="center" justifyContent="space-between">
                    <Avatar sx={{ bgcolor: selectColor(client.hash) }}>{client.id.substring(0, 2)}</Avatar>
                    <span>Links: {messageCount}</span>
                    <Button
                      onClick={() => onUpdate?.(client.id, faker.word.noun(), faker.word.noun())}
                      variant="outlined"
                      endIcon={<AddIcon />}
                    >
                      Add Data
                    </Button>
                  </Stack>
                  <JsonViewer
                    editable
                    onChange={([k], _, v) => onUpdate?.(client.id, k.toString(), v + '')}
                    value={Object.fromEntries(simulator.debug_client(client.id).entries())}
                    style={{ flexGrow: 1 }}
                    highlightUpdates
                  />
                </Stack>
              </Card>
            </Box>
          )
        }}
      />
    )
  }
)
