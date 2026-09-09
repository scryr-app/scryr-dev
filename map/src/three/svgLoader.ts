import { SVGLoader } from "three/examples/jsm/loaders/SVGLoader.js";

/**
 * Keep SVG-rendering dependencies compatible with Three.js r185+.
 *
 * UIKit still calls the deprecated static helper. Redirect it to the ShapePath
 * API that Three.js now exposes directly so SVG rendering does not emit a
 * deprecation warning.
 */
SVGLoader.createShapes = (shapePath) => shapePath.toShapes();
