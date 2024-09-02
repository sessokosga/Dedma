# Dedma
[Read in english](README.md)
### Générateur de notes de versions (Release notes)
Un outil en ligne de commande qui converti vos dernier commit en notes de version.

## Afficher l'aide
    dedma --help

## Spécifier la langue dans la laquelle générer les commit
### Français
Pour générer une note en français, ajouter `LANG_FR` dans les variables d'environnement.
Pour les systèmes basés sur Unix
	LANG_FR=1 

Pour Windows PowerShell
	$Env:LANG_FR=1

### Anglais
La note est générée en anglais par défaut

## Générer des notes à partir des commit dans un fichier
    dedma fichier_d_entree fichier_de_sortie

## Générer les notes à partir des commit de Git
    dedma fichier_de_sortie
    
ou 

    dedma

pour générer les notes dans le fichier `whats_new.md`

## Exemple
Le programme converti ceci 

    feat (Tour): Ajout d'un nouveau type de Tour 
    fix: Rendu la rotation et le point de départ des projectiles lié à la position du tank 
    feat (Récompenses): Ajout de deux récompenses supplémentaire
    update: Equilibrage du jeu
    feat (Economie): Ajout d'un système de monétaire

En ceci :

    # Nouvelles fonctionnalités
    ## Tour
    - Ajout d'un nouveau type de Tour 
    ## Récompenses
    -  Ajout de deux récompenses supplémentaire
    ## Economie
    - monétaire
    # Correction d'erreur
    - Rendu la rotation et le point de départ des projectiles lié à la position du tank 
    # Mise à jour
    - Equilibrage du jeu

## Structure de commit idéale
    type (titre): contenu
Le `titre` et le `contenu` peuvent être ce que vous voulez.  
Voici une liste des types supportés actuellement, classés par ordre d'apparence dans la note générées.


|   type   |        nom complet        | signification                                                                                                                                               |
| :------: | :-----------------------: | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
|   feat   | Nouvelles fonctionnalités | Une nouvelle fonctionnalité a été ajoutée                                                                                                                   |
|   fix    |    Correction d'erreur    | Une erreur a été corrigé                                                                                                                                    |
|  chore   |           Chore           | Modification qui ne modifie pas le code source, ni les test. Par exemple la mise à jour des dépendances.                                                    |
| refactor |        Refactoring        | Amélioration du code, qui ne corrige pas un bug et n'ajoute pas de nouvelles fonctionnalités                                                                |
|   docs   |       Documentation       | Modification de la documentation comme le fichier README ou d'autres fichier markdown                                                                       |
|  style   |       Style de Code       | Modification qui n'affectent pas la signification du code, qui a plutôt lien avec le formatage comme les espace, un point-virgule manquant, ainsi de suite. |
|   test   |           Test            | Ajout ou modification de test                                                                                                                               |
|   perf   |       Performances        | amélioration de performances                                                                                                                                |
|    ci    |   Déploiements Continue    | Ce qui a lien avec le déploiement continue                                                                                                                  |
|  build   |     Système de Build      | Modification qui affecte le système de compilation ou des dépendences externes                                                                              |
|  revert  |          Annulations           | Annulation d'un commit                                                                                                                                      |
|  update  |        Mise à jour        | Toute mise à jour                                                                                                                                           |