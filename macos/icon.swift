// draws Zakuro's icon, a pomegranate (zakuro) on a dark plum tile, as a
// 1024 pixel PNG. usage: swift macos/icon.swift out.png
import AppKit

let size = 1024
let bitmap = NSBitmapImageRep(
    bitmapDataPlanes: nil, pixelsWide: size, pixelsHigh: size, bitsPerSample: 8, samplesPerPixel: 4,
    hasAlpha: true, isPlanar: false, colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: bitmap)

func color(_ hex: Int, _ alpha: CGFloat = 1) -> NSColor {
    NSColor(
        red: CGFloat((hex >> 16) & 0xff) / 255, green: CGFloat((hex >> 8) & 0xff) / 255,
        blue: CGFloat(hex & 0xff) / 255, alpha: alpha)
}

// the tile, in the 824 point square macOS leaves inside the canvas
let tile = NSBezierPath(roundedRect: NSRect(x: 100, y: 100, width: 824, height: 824), xRadius: 185, yRadius: 185)
NSGraphicsContext.saveGraphicsState()
let tileShadow = NSShadow()
tileShadow.shadowOffset = NSSize(width: 0, height: -12)
tileShadow.shadowBlurRadius = 24
tileShadow.shadowColor = color(0x000000, 0.35)
tileShadow.set()
color(0x2a1424).setFill()
tile.fill()
NSGraphicsContext.restoreGraphicsState()
NSGradient(starting: color(0x4a2238), ending: color(0x1a0b14))!.draw(in: tile, angle: -90)

// the fruit
let center = NSPoint(x: 512, y: 480)
let radius: CGFloat = 270
let fruit = NSBezierPath(ovalIn: NSRect(x: center.x - radius, y: center.y - radius, width: radius * 2, height: radius * 2))
NSGraphicsContext.saveGraphicsState()
let fruitShadow = NSShadow()
fruitShadow.shadowOffset = NSSize(width: 0, height: -18)
fruitShadow.shadowBlurRadius = 36
fruitShadow.shadowColor = color(0x000000, 0.5)
fruitShadow.set()
color(0xc8203a).setFill()
fruit.fill()
NSGraphicsContext.restoreGraphicsState()
NSGradient(colors: [color(0xff6f6f), color(0xe0243c), color(0x8f0f24)], atLocations: [0, 0.5, 1], colorSpace: .deviceRGB)!
    .draw(in: fruit, relativeCenterPosition: NSPoint(x: -0.3, y: 0.4))

// the crown the fruit wears at the top
let crown = NSBezierPath()
crown.move(to: NSPoint(x: 445, y: 730))
for (x, y) in [(428, 825), (482, 775), (512, 842), (542, 775), (596, 825), (579, 730)] {
    crown.line(to: NSPoint(x: x, y: y))
}
crown.close()
NSGradient(starting: color(0xc8203a), ending: color(0x7a0c1c))!.draw(in: crown, angle: -90)

// a soft highlight
NSGraphicsContext.saveGraphicsState()
fruit.addClip()
color(0xffffff, 0.22).setFill()
NSBezierPath(ovalIn: NSRect(x: 330, y: 540, width: 190, height: 130)).fill()
NSGraphicsContext.restoreGraphicsState()

NSGraphicsContext.restoreGraphicsState()
try! bitmap.representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: CommandLine.arguments[1]))
