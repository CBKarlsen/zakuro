// the app around a game Zakuro runs: the two screens, the console's buttons
// and the touch screen. Zakuro itself is in crates/zakuro-apple.

import SwiftUI
import UniformTypeIdentifiers

/// the console's button bits, as zakuro_set_buttons takes them.
enum Pad {
    static let a: UInt32 = 1 << 0
    static let b: UInt32 = 1 << 1
    static let select: UInt32 = 1 << 2
    static let start: UInt32 = 1 << 3
    static let right: UInt32 = 1 << 4
    static let left: UInt32 = 1 << 5
    static let up: UInt32 = 1 << 6
    static let down: UInt32 = 1 << 7
    static let r: UInt32 = 1 << 8
    static let l: UInt32 = 1 << 9
    static let x: UInt32 = 1 << 10
    static let y: UInt32 = 1 << 11

    /// the buttons held, changed from the main thread by the touches on them.
    private static var held: UInt32 = 0

    static func set(_ bit: UInt32, down: Bool) {
        held = down ? held | bit : held & ~bit
        zakuro_set_buttons(held)
    }
}

/// a screen's picture, which the game's thread puts a new one in.
final class PictureView: UIView {
    func show(_ image: CGImage) {
        DispatchQueue.main.async { self.layer.contents = image }
    }
}

struct Picture: UIViewRepresentable {
    let view: PictureView

    func makeUIView(context: Context) -> PictureView { view }
    func updateUIView(_ view: PictureView, context: Context) {}
}

/// the game, run on a thread of its own, one frame to every sixtieth of a second.
final class Emulator: ObservableObject {
    enum State {
        case idle, running, failed(String), ended
    }

    @Published var state = State.idle
    let top = PictureView()
    let bottom = PictureView()

    func start(rom: String) {
        // the recompiled code uses the stack as the console's code does, and a thread has little by default
        let thread = Thread { self.run(rom: rom) }
        thread.stackSize = 64 << 20
        thread.start()
    }

    private func run(rom: String) {
        let id = Bundle.main.object(forInfoDictionaryKey: "ZakuroProgramID") as? String ?? ""
        let saves = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0].appendingPathComponent("zakuro").path
        guard let programID = UInt64(id, radix: 16), let game = zakuro_open(rom, saves, programID, 2) else {
            finish(.failed("Could not open the game."))
            return
        }
        DispatchQueue.main.async { self.state = .running }
        var next = CACurrentMediaTime()
        while zakuro_run_frame(game) {
            show(game, screen: 0, in: top)
            show(game, screen: 1, in: bottom)
            next += 1.0 / 60
            let wait = next - CACurrentMediaTime()
            if wait > 0 { Thread.sleep(forTimeInterval: wait) } else { next = CACurrentMediaTime() }
        }
        zakuro_close(game)
        finish(.ended)
    }

    private func finish(_ state: State) {
        DispatchQueue.main.async { self.state = state }
    }

    private func show(_ game: OpaquePointer, screen: UInt32, in view: PictureView) {
        var width: UInt32 = 0, height: UInt32 = 0
        guard let pixels = zakuro_screen(game, screen, &width, &height),
              let provider = CGDataProvider(data: Data(bytes: pixels, count: Int(width * height * 4)) as CFData),
              let image = CGImage(
                width: Int(width), height: Int(height), bitsPerComponent: 8, bitsPerPixel: 32, bytesPerRow: Int(width * 4),
                space: CGColorSpaceCreateDeviceRGB(), bitmapInfo: CGBitmapInfo(rawValue: CGImageAlphaInfo.noneSkipLast.rawValue),
                provider: provider, decode: nil, shouldInterpolate: true, intent: .defaultIntent)
        else { return }
        view.show(image)
    }
}

/// a button of the console, held while a finger is on it.
struct HoldButton: View {
    let label: String
    let bit: UInt32
    var width: CGFloat = 52
    @State private var down = false

    var body: some View {
        Text(label)
            .font(.system(size: 16, weight: .semibold))
            .frame(width: width, height: 44)
            .background(down ? Color.accentColor : Color.white.opacity(0.18), in: RoundedRectangle(cornerRadius: 10))
            .foregroundStyle(.white)
            .gesture(DragGesture(minimumDistance: 0)
                .onChanged { _ in
                    if !down { down = true; Pad.set(bit, down: true) }
                }
                .onEnded { _ in
                    down = false
                    Pad.set(bit, down: false)
                })
    }
}

struct Controls: View {
    var body: some View {
        VStack(spacing: 10) {
            HStack {
                HoldButton(label: "L", bit: Pad.l, width: 90)
                Spacer()
                HoldButton(label: "R", bit: Pad.r, width: 90)
            }
            HStack(alignment: .center) {
                VStack(spacing: 4) {
                    HoldButton(label: "▲", bit: Pad.up)
                    HStack(spacing: 4) {
                        HoldButton(label: "◀", bit: Pad.left)
                        HoldButton(label: "▶", bit: Pad.right)
                    }
                    HoldButton(label: "▼", bit: Pad.down)
                }
                Spacer()
                VStack(spacing: 10) {
                    HoldButton(label: "Start", bit: Pad.start, width: 70)
                    HoldButton(label: "Select", bit: Pad.select, width: 70)
                }
                Spacer()
                VStack(spacing: 4) {
                    HoldButton(label: "X", bit: Pad.x)
                    HStack(spacing: 4) {
                        HoldButton(label: "Y", bit: Pad.y)
                        HoldButton(label: "A", bit: Pad.a)
                    }
                    HoldButton(label: "B", bit: Pad.b)
                }
            }
        }
        .padding(.horizontal, 20)
    }
}

struct GameView: View {
    let emulator: Emulator

    var body: some View {
        VStack(spacing: 8) {
            Picture(view: emulator.top)
                .aspectRatio(400.0 / 240.0, contentMode: .fit)
            GeometryReader { size in
                Picture(view: emulator.bottom)
                    .gesture(DragGesture(minimumDistance: 0)
                        .onChanged { touch in
                            let x = min(max(touch.location.x / size.size.width * 320, 0), 319)
                            let y = min(max(touch.location.y / size.size.height * 240, 0), 239)
                            zakuro_set_touch(UInt16(x), UInt16(y), false)
                        }
                        .onEnded { _ in zakuro_set_touch(0, 0, true) })
            }
            .aspectRatio(320.0 / 240.0, contentMode: .fit)
            Controls()
            Spacer(minLength: 0)
        }
    }
}

struct ContentView: View {
    @StateObject private var emulator = Emulator()
    @State private var choosing = false

    var body: some View {
        ZStack {
            Color.black.ignoresSafeArea()
            switch emulator.state {
            case .idle:
                Button("Choose a game") { choosing = true }
                    .buttonStyle(.borderedProminent)
            case .running:
                GameView(emulator: emulator)
            case .failed(let message):
                Text(message).foregroundStyle(.white)
            case .ended:
                Text("The game ended.").foregroundStyle(.white)
            }
        }
        .fileImporter(isPresented: $choosing, allowedContentTypes: [.data]) { result in
            if let url = try? result.get(), let copy = copied(url) { emulator.start(rom: copy.path) }
        }
        .onAppear {
            // a game named for a test run, or one put in the app's folder from Files
            if let rom = ProcessInfo.processInfo.environment["ZK_ROM"] ?? gamesInFolder().first {
                emulator.start(rom: rom)
            }
        }
    }

    private var documents: URL { FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0] }

    private func gamesInFolder() -> [String] {
        let files = (try? FileManager.default.contentsOfDirectory(at: documents, includingPropertiesForKeys: nil)) ?? []
        return files.filter { ["3ds", "cci", "cxi", "cia"].contains($0.pathExtension.lowercased()) }.map(\.path)
    }

    /// a game picked outside the app, copied into it, where it can be read.
    private func copied(_ url: URL) -> URL? {
        let access = url.startAccessingSecurityScopedResource()
        defer { if access { url.stopAccessingSecurityScopedResource() } }
        let copy = documents.appendingPathComponent(url.lastPathComponent)
        try? FileManager.default.removeItem(at: copy)
        return (try? FileManager.default.copyItem(at: url, to: copy)) == nil ? nil : copy
    }
}

@main
struct ZakuroApp: App {
    var body: some Scene {
        WindowGroup { ContentView() }
    }
}
