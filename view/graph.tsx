import * as PIXI from 'pixi.js'
import { OutlineFilter } from '@pixi/filter-outline'
import React, { useEffect, useLayoutEffect, useRef } from 'react'
import { ViewState } from '.'

const devicePixelRatio = window.devicePixelRatio || 1;
const appStageSize = 300;
const clientRectSize = 25;

const outlineFilterBlue = new OutlineFilter(1 * devicePixelRatio, 0x3949ab)
const outlineFilterWrite = new OutlineFilter(1 * devicePixelRatio, 0xe78ac3)

const termColors = [
  '#66c2a5',
  '#fc8d62',
  '#8da0cb',
  '#e78ac3',
  '#a6d854',
  '#ffd92f',
]

export function selectColor(a: string) {
  let t = 0
  for (let i = 0; i < a.length; i++) {
    t += a.charCodeAt(i)
  }
  return termColors[t % termColors.length | 0]
}

interface GraphState extends ViewState {
  selectId?: string | null
}

function createPIXIApp(target: HTMLElement, onSelect?: (id: string, index: number) => void) {
  const app = new PIXI.Application({
    background: '#fff',
    width: appStageSize * devicePixelRatio,
    height: appStageSize * devicePixelRatio,
    antialias: true,
  })
  app.stage.interactive = true
  app.stage.hitArea = app.screen
  app.stage.sortableChildren = true

  app.stage.scale = {
    x: devicePixelRatio,
    y: devicePixelRatio,
  }
  app.view.style!.width = appStageSize + 'px'
  app.view.style!.height = appStageSize + 'px'
  target.appendChild(app.view as any)

  let dragTarget: PIXI.DisplayObject | null = null
  let dragOffset: PIXI.IPointData

  function handleDragMove(event: PIXI.FederatedPointerEvent) {
    if (dragTarget) {
      dragTarget.parent.toLocal(
        { x: event.globalX + dragOffset.x, y: event.globalY + dragOffset.y },
        undefined,
        dragTarget.position
      )
    }
  }

  function handleDragEnd() {
    if (dragTarget) {
      app.stage.off('pointermove', handleDragMove)
      app.stage.off('pointerup', handleDragEnd)
      app.stage.off('pointerupoutside', handleDragEnd)
      dragTarget.zIndex = 1
      dragTarget = null
    }
  }

  function handleDragStart(
    event: PIXI.FederatedPointerEvent,
    rect: PIXI.DisplayObject
  ) {
    rect.zIndex = 100
    dragTarget = rect
    dragOffset = {
      x: rect.getGlobalPosition().x - event.globalX,
      y: rect.getGlobalPosition().y - event.globalY,
    }
    app.stage.on('pointermove', handleDragMove)
    app.stage.on('pointerup', handleDragEnd)
    app.stage.on('pointerupoutside', handleDragEnd)
  }

  function handlePointerEnterRect(rect: PIXI.DisplayObject) {
    rect.filters = [outlineFilterWrite]
  }

  function handlePointerLeaveRect(rect: PIXI.DisplayObject) {
    rect.filters = []
  }

  let rects: {
    container: PIXI.Container
    rect: PIXI.Graphics
    text: PIXI.Text
  }[] = []

  // Rectangle
  function renderRect(state: GraphState) {
    for (let i = 0; i < state.clients.length; i++) {
      const client = state.clients[i]
      let r
      if (rects[i]) {
        r = rects[i]
        r.rect.clear()
      } else {
        const container = new PIXI.Container()
        const rect = new PIXI.Graphics()
        const text = new PIXI.Text()
        container.addChild(rect)
        container.addChild(text)
        rects[i] = { container, rect, text }
        r = rects[i]
        app.stage.addChild(container)
        container.x =
          Math.random() * (appStageSize * 0.75) + appStageSize * 0.125 - clientRectSize / 2
        container.y =
          Math.random() * (appStageSize * 0.75) + appStageSize * 0.125 - clientRectSize / 2
        container.interactive = true
        container.cursor = 'pointer'
        container.zIndex = 1
        container.on('pointerdown', (e) => handleDragStart(e, container))
        container.on('pointerenter', () => handlePointerEnterRect(container))
        container.on('pointerleave', () => handlePointerLeaveRect(container))
        container.on('pointertap', () => onSelect?.(client.id, i))
        container.filters = []
      }

      r.rect.beginFill(selectColor(client.hash))
      r.rect.lineStyle(1 * devicePixelRatio, 0x9c27b0)
      r.rect.drawRect(0, 0, clientRectSize, clientRectSize)
      r.rect.endFill()
      r.rect.x = 0
      r.rect.y = 0
      r.text.text = client.id.slice(0, 2)
      r.text.anchor.set(0.5, 0.5)
      r.text.x = clientRectSize / 2
      r.text.y = clientRectSize / 2
      r.text.scale = { x: 0.5, y: 0.5 }
      r.text.resolution = devicePixelRatio
    }

    rects
      .splice(state.clients.length)
      .map((r) => r.container.removeFromParent())
  }

  let pathes: PIXI.Graphics[] = []

  function renderPath(state: GraphState) {
    let i = 0
    for (let from = 0; from < rects.length; from++) {
      const fromRect = rects[from]
      const fromClient = state.clients[from]

      for (let to = from + 1; to < rects.length; to++) {
        const toRect = rects[to]
        const toClient = state.clients[to]
        let path
        if (pathes[i]) {
          path = pathes[i]
          path.clear()
        } else {
          path = new PIXI.Graphics()
          pathes[i] = path
          app.stage.addChild(path)
        }
        i++
        if (
          !state.messages.find(
            (m) =>
              (m.to === toClient.id && m.from === fromClient.id) ||
              (m.to === fromClient.id && m.from === toClient.id)
          )
        ) {
          continue
        }
        path.lineStyle(1 * devicePixelRatio, 0x333333, 0.5)
        const fromPoint = new PIXI.Point(
          fromRect.container.x + fromRect.container.getLocalBounds().width / 2,
          fromRect.container.y + fromRect.container.getLocalBounds().height / 2
        )
        const toPoint = new PIXI.Point(
          toRect.container.x + toRect.container.getLocalBounds().width / 2,
          toRect.container.y + toRect.container.getLocalBounds().height / 2
        )
        path.moveTo(fromPoint.x, fromPoint.y)
        path.lineTo(toPoint.x, toPoint.y)
        path.x = 0
        path.y = 0
        path.zIndex = -1
      }
    }
  }

  let messagePointes: PIXI.Graphics[] = []

  function renderMessage(state: GraphState) {
    let i = 0
    for (const message of state.messages) {
      const fromRect =
        rects[state.clients.findIndex((c) => c.id === message.from)]
      const toRect = rects[state.clients.findIndex((c) => c.id === message.to)]

      const fromPoint = new PIXI.Point(
        fromRect.container.x + fromRect.container.getLocalBounds().width / 2,
        fromRect.container.y + fromRect.container.getLocalBounds().height / 2
      )
      const toPoint = new PIXI.Point(
        toRect.container.x + toRect.container.getLocalBounds().width / 2,
        toRect.container.y + toRect.container.getLocalBounds().height / 2
      )

      const messagePoint = {
        x:
          fromPoint.x > toPoint.x
            ? fromPoint.x - (fromPoint.x - toPoint.x) * message.progress
            : fromPoint.x + (toPoint.x - fromPoint.x) * message.progress,
        y:
          fromPoint.y > toPoint.y
            ? fromPoint.y - (fromPoint.y - toPoint.y) * message.progress
            : fromPoint.y + (toPoint.y - fromPoint.y) * message.progress,
      }

      let point
      if (messagePointes[i]) {
        point = messagePointes[i]
        point.clear()
      } else {
        point = new PIXI.Graphics()
        messagePointes[i] = point
        app.stage.addChild(point)
      }
      i++

      point.lineStyle(1)
      point.beginFill(selectColor(message.kind), 1)
      point.drawCircle(0, 0, 5)
      point.endFill()
      point.zIndex = 0
      point.filters = [outlineFilterBlue]
      point.position.x = messagePoint.x
      point.position.y = messagePoint.y
    }

    messagePointes
      .splice(state.messages.length)
      .map((r) => r.removeFromParent())
  }

  return {
    render(state: GraphState) {
      renderRect(state)
      renderMessage(state)
      renderPath(state)
    },
    destroy() {
      app.destroy(true)
    },
  }
}

export const Graph = ({
  state,
  onSelect,
}: {
  state: GraphState
  onSelect?: (id: string, index: number) => void
}) => {
  const divElement = useRef<HTMLDivElement>(null)
  const appRef = useRef<ReturnType<typeof createPIXIApp>>()

  useLayoutEffect(() => {
    if (divElement.current) {
      const app = createPIXIApp(divElement.current, onSelect)
      appRef.current = app

      return () => {
        app.destroy()
      }
    }
  }, [])

  useEffect(() => {
    appRef.current?.render(state)
  }, [state])

  return <div ref={divElement}></div>
}
