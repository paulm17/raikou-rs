// Prints on-screen, layer-0 windows whose owner or title contains the given
// substring. One window per line: <id>|<owner>|<title>|x,y,w,h
import CoreGraphics
import Foundation

let target = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : ""
let opts: CGWindowListOption = [.optionOnScreenOnly, .excludeDesktopElements]
guard let list = CGWindowListCopyWindowInfo(opts, kCGNullWindowID) as? [[String: Any]] else {
    exit(1)
}
for w in list {
    let owner = w[kCGWindowOwnerName as String] as? String ?? ""
    let layer = w[kCGWindowLayer as String] as? Int ?? -1
    guard layer == 0 else { continue }
    let name = w[kCGWindowName as String] as? String ?? ""
    guard owner.localizedCaseInsensitiveContains(target)
        || name.localizedCaseInsensitiveContains(target) else { continue }
    let id = w[kCGWindowNumber as String] as? Int ?? 0
    let b = w[kCGWindowBounds as String] as? [String: Any] ?? [:]
    let x = b["X"] as? Int ?? 0
    let y = b["Y"] as? Int ?? 0
    let width = b["Width"] as? Int ?? 0
    let height = b["Height"] as? Int ?? 0
    print("\(id)|\(owner)|\(name)|\(x),\(y),\(width),\(height)")
}
