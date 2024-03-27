# Nebulous Fleet Generator

Work-in-progress random fleet generator and calculator for NEBULOUS: Fleet Command.

This repository is in three parts:
- `nebulous-data`: Contains lists of all the components, hulls, and munitions in
  the game as well as structures for serializing and deserializing fleet files.
- `nebulous-fleet-duplicator`: Takes a fleet file, producing a new fleet that has
  exactly twice as many ships in it (for when you want to double a fleet for 2v1's or such).
- `nebulous-fleet-generator`: The generator and calculator logic itself.
- `nebulous-xml`: Framework for serializing/deserializing xml.
